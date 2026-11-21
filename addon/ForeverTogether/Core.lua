-- Reads the player's character from the client and builds the code. No timers, no polling: events only mark the code stale.
-- Sources and fallbacks: docs/rdf-code-spec.md, "Collecting each field in the addon". Provisional until spike S0.
local ADDON, NS = ...
local RD = NS.RD

RD.counter = 0 -- raised on every build in this session, wraps from 65535 to 0 (not saved)
RD.stale = true -- true until a code has been built, and again whenever something the code carries may have changed
RD.last = nil -- { text =, fields =, created = (server time, seconds), missing = {...} } of the latest successful build

local ROLE = { TANK = 1, HEALER = 2, DAMAGER = 3 }
local EQUIP_SLOTS = { 1, 2, 3, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18 } -- no shirt, no tabard

local function call(fn, ...)
  if type(fn) ~= "function" then return nil end
  local ok, a, b, c, d, e = pcall(fn, ...)
  if ok then return a, b, c, d, e end
end

-- First name and surname: UnitName("player") is assumed to give "First Surname"; if not, GetUnitName("player") is split on
-- the first space. The realm part (second return) is never read: on Forever it is a surname (ADR 0022).
local function splitName()
  for _, getter in ipairs({ UnitName, GetUnitName }) do
    local full = call(getter, "player")
    if type(full) == "string" then
      local first, rest = full:match("^(%S+)%s+(.+)$")
      if first then return first, rest end
    end
  end
  return nil
end

-- Item level fallback: slot sum, taken only when PLAYER_EQUIPMENT_CHANGED fires and GetAverageItemLevel is missing.
local function sumSlots()
  local total, count, mainLevel = 0, 0, 0
  for _, slot in ipairs(EQUIP_SLOTS) do
    local link = call(GetInventoryItemLink, "player", slot)
    local level = 0
    if link and C_Item and C_Item.GetDetailedItemLevelInfo then level = call(C_Item.GetDetailedItemLevelInfo, link) or 0 end
    if slot == 16 then mainLevel = level end
    if slot == 17 and level == 0 then level = mainLevel end -- a two-hander fills both hand slots
    total, count = total + level, count + 1
  end
  if count == 0 or total == 0 then return 0 end
  return math.floor(total / count + 0.5) -- mean over the 17 listed slots, a two-hander counted in both hands (formula unverified, S0)
end
RD.SumSlots = sumSlots

local function realmValue()
  local id = C_GameRules and call(C_GameRules.GetCurrentGameModeRecordID)
  if type(id) == "number" then return tostring(math.floor(id)) end
  local name = call(GetRealmName)
  if type(name) == "string" and name ~= "" then return (name:gsub("%u", string.lower)) end -- ASCII-only lowercase
end

-- Returns fields, missing where missing lists the unknown values for the diagnostic line ("region=?"). When the code cannot
-- be built the first return is nil and the second is a one-line reason; `missing` is still returned as the third value.
function RD.Collect()
  local missing = {}
  local fields = { secondaryRole = 0 }

  local first, surname = splitName()
  if not first then
    missing[#missing + 1] = "name=?"
  else
    fields.firstName = RD.TruncateUtf8(first, RD.MAX_NAME)
    fields.surname = RD.TruncateUtf8(surname, RD.MAX_NAME)
  end

  local realm = realmValue()
  if realm then fields.realm = RD.TruncateUtf8(realm, RD.MAX_REALM) else missing[#missing + 1] = "realm=?" end

  local region = call(GetCurrentRegion)
  if type(region) ~= "number" then missing[#missing + 1] = "region=?"; region = 0 end
  fields.region = math.max(0, math.min(255, math.floor(region)))

  local faction = call(UnitFactionGroup, "player")
  faction = type(faction) == "string" and faction:lower() or nil
  if faction == "alliance" or faction == "horde" then fields.faction = faction else missing[#missing + 1] = "faction=?" end

  local _, _, classID = call(UnitClass, "player")
  local _, _, raceID = call(UnitRace, "player")
  fields.class = type(classID) == "number" and math.max(0, math.min(255, classID)) or 0
  fields.race = type(raceID) == "number" and math.max(0, math.min(255, raceID)) or 0
  if fields.class == 0 then missing[#missing + 1] = "class=?" end
  if fields.race == 0 then missing[#missing + 1] = "race=?" end

  local level = call(UnitLevel, "player")
  if type(level) == "number" and level >= 1 then fields.level = math.min(255, math.floor(level)) else missing[#missing + 1] = "level=?" end

  local spec = C_SpecializationInfo or {}
  local getSpec = spec.GetSpecialization or _G.GetSpecialization -- the client exports it under C_SpecializationInfo
  local getInfo = spec.GetSpecializationInfo or _G.GetSpecializationInfo
  local specIndex = call(getSpec)
  fields.spec, fields.role = 0, 0
  if specIndex then
    local id, _, _, _, role = call(getInfo, specIndex)
    if type(id) == "number" then fields.spec = math.max(0, math.min(65535, math.floor(id))) end
    fields.role = ROLE[role] or 0
  end
  if fields.spec == 0 then missing[#missing + 1] = "spec=?" end

  local itemLevel = call(GetAverageItemLevel)
  if type(itemLevel) ~= "number" then itemLevel = RD.fallbackItemLevel end
  if type(itemLevel) == "number" and itemLevel >= 1 then fields.itemLevel = math.min(65535, math.floor(itemLevel + 0.5))
  else fields.itemLevel = 0; missing[#missing + 1] = "itemLevel=?" end

  local zone = call(GetRealZoneText)
  if type(zone) ~= "string" or zone == "" then zone = call(GetZoneText) end
  fields.zone = type(zone) == "string" and RD.TruncateUtf8(zone, RD.MAX_ZONE) or ""
  if fields.zone == "" then missing[#missing + 1] = "zone=?" end

  fields.created = math.floor((call(GetServerTime) or 0) / 60)
  fields.counter = 0

  -- Values the code cannot do without. Region, class, race, spec, role, item level and zone fall back to "unknown".
  local fatal = {}
  if not fields.firstName then fatal[#fatal + 1] = "name" end
  if not fields.realm then fatal[#fatal + 1] = "realm" end
  if not fields.faction then fatal[#fatal + 1] = "faction" end
  if not fields.level then fatal[#fatal + 1] = "level" end
  if #fatal > 0 then return nil, "cannot build the code: " .. table.concat(fatal, ", ") .. " unknown", missing end
  return fields, missing
end

-- Builds a new code and raises the counter. Returns text, fields, created (server time in seconds), missing; or nil, reason.
function RD.BuildCode()
  local fields, missing, diag = RD.Collect() -- on failure: nil, reason, missing list
  if not fields then return nil, missing, diag end
  RD.counter = (RD.counter + 1) % 65536
  fields.counter = RD.counter
  local text, err = RD.ToText(fields)
  if not text then return nil, "cannot build the code: " .. tostring(err), missing end
  RD.stale = false
  RD.last = { text = text, fields = fields, created = (call(GetServerTime) or 0), missing = missing }
  return text, fields, RD.last.created, missing
end

-- Events only mark the code stale; the window offers a Refresh.
local frame = CreateFrame("Frame")
RD.eventFrame = frame
for _, event in ipairs({ "PLAYER_LEVEL_UP", "ZONE_CHANGED_NEW_AREA", "PLAYER_ENTERING_WORLD", "PLAYER_EQUIPMENT_CHANGED",
  "PLAYER_SPECIALIZATION_CHANGED", "ACTIVE_TALENT_GROUP_CHANGED" }) do
  frame:RegisterEvent(event)
end
frame:SetScript("OnEvent", function(_, event)
  RD.stale = true
  if event == "PLAYER_EQUIPMENT_CHANGED" and type(GetAverageItemLevel) ~= "function" then RD.fallbackItemLevel = sumSlots() end
  if RD.OnStale then RD.OnStale(event) end
end)
