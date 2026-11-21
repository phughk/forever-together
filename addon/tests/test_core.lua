-- Core: Collect follows the table in docs/rdf-code-spec.md; BuildCode raises the counter; events only mark the code stale.
local V = vm.read_json("docs/rdf-code-vectors.json")
local function RD() return ForeverTogether.RD end
local player = vm.world.player

-- Run fn with player fields (or globals) temporarily changed; everything is put back afterwards.
local function with(changes, fn, globals)
  local saved, savedG = {}, {}
  for k, v in pairs(changes or {}) do saved[k] = player[k]; if v == false then player[k] = nil else player[k] = v end end
  for k, v in pairs(globals or {}) do savedG[k] = rawget(_G, k); _G[k] = v end
  local ok, err = pcall(fn)
  for k in pairs(changes or {}) do player[k] = saved[k] end
  for k in pairs(globals or {}) do _G[k] = savedG[k] end
  if not ok then error(err, 0) end
end

local function typical()
  for _, c in ipairs(V.records) do if c.name == "typical record" then return c end end
end

vm.test("Collect reads the character: two-field name, level, faction, spec, role, item level, zone, region", function()
  local f, missing = RD().Collect()
  vm.assert_true(f, tostring(missing))
  vm.assert_eq(f.firstName, "Thalion")
  vm.assert_eq(f.surname, "Brightwood")
  vm.assert_eq(f.level, 24)
  vm.assert_eq(f.faction, "alliance")
  vm.assert_eq(f.class, 1) -- Warrior
  vm.assert_eq(f.spec, 73)
  vm.assert_eq(f.role, 1) -- TANK
  vm.assert_eq(f.secondaryRole, 0)
  vm.assert_eq(f.itemLevel, 412)
  vm.assert_eq(f.zone, "Westfall")
  vm.assert_eq(f.region, 3)
  vm.assert_eq(f.realm, "mankrik") -- fallback: the realm name lowercased, until spike S0 names the ruleset call
  vm.assert_eq(f.created, math.floor(GetServerTime() / 60))
end)

vm.test("the realm comes from the game-mode record ID when the client has one, as a decimal string", function()
  with({}, function()
    local f = RD().Collect()
    vm.assert_eq(f.realm, "8")
  end, { C_GameRules = { GetCurrentGameModeRecordID = function() return 8 end } })
end)

vm.test("a built code decodes to what Collect read and matches the typical vector when the same values are used", function()
  with({}, function()
    local before = RD().counter
    local text, fields = RD().BuildCode()
    vm.assert_true(text, tostring(fields))
    vm.assert_eq(RD().counter, before + 1)
    vm.assert_eq(fields.counter, before + 1)
    local back = RD().FromText(text)
    for k, v in pairs(fields) do vm.assert_eq(back[k], v, "round trip field " .. k) end
    -- same inputs as the "typical record" vector (race ID is not in the VM, so it is supplied here)
    local c = typical()
    RD().counter = c.fields.counter - 1
    local t2, f2 = RD().BuildCode()
    vm.assert_eq(f2.counter, c.fields.counter)
    local ref = c.fields
    f2.created = ref.created
    f2.race = ref.race
    for _, k in ipairs({ "role", "faction", "itemLevel", "class", "level", "spec", "region", "realm", "firstName", "surname", "zone" }) do
      vm.assert_eq(f2[k], ref[k], "typical vector field " .. k)
    end
  end, { C_GameRules = { GetCurrentGameModeRecordID = function() return 3 end } })
end)

vm.test("counter wraps from 65535 to 0", function()
  RD().counter = V.counter_wrap.from
  local text, fields = RD().BuildCode()
  vm.assert_true(text)
  vm.assert_eq(fields.counter, V.counter_wrap.to)
  vm.assert_eq(RD().FromText(text).counter, 0)
  vm.assert_eq(RD().counter, 0)
end)

vm.test("each role maps from the spec (tank 1, healer 2, damage 3) and an unknown spec gives spec 0, role 0", function()
  with({ specID = 256 }, function() vm.assert_eq(RD().Collect().role, 2); vm.assert_eq(RD().Collect().spec, 256) end)
  with({ specID = 999 }, function() vm.assert_eq(RD().Collect().role, 3) end) -- unlisted spec: the VM defaults to DAMAGER
  with({ specID = false }, function()
    local f = RD().Collect()
    vm.assert_eq(f.spec, 0); vm.assert_eq(f.role, 0)
  end)
end)

vm.test("item level unknown: 0, with the code still built", function()
  with({ itemLevel = false }, function()
    local f, missing = RD().Collect()
    vm.assert_eq(f.itemLevel, 0)
    local found = false
    for _, m in ipairs(missing) do if m == "itemLevel=?" then found = true end end
    vm.assert_true(found, "diagnostic lists itemLevel=?")
    local text = RD().BuildCode()
    vm.assert_eq(RD().FromText(text).itemLevel, 0)
  end)
end)

vm.test("Horde faction, and a missing faction refuses to build with a reason", function()
  with({ faction = "Horde" }, function() vm.assert_eq(RD().Collect().faction, "horde") end)
  with({ faction = "Neutral" }, function()
    local f, why = RD().Collect()
    vm.assert_true(f == nil and why:find("faction"), tostring(why))
    local text, reason = RD().BuildCode()
    vm.assert_true(text == nil and reason:find("faction"))
  end)
end)

vm.test("the name is two fields: a name with no space falls back to GetUnitName and then refuses", function()
  with({ name = "Thalion" }, function()
    local f, why = RD().Collect()
    vm.assert_true(f == nil and why:find("name"), tostring(why))
  end)
  with({ name = "Thalion" }, function()
    local f = RD().Collect()
    vm.assert_eq(f.firstName, "Thalion")
    vm.assert_eq(f.surname, "Brightwood")
  end, { UnitName = function() return "Thalion" end, GetUnitName = function() return "Thalion Brightwood" end })
end)

vm.test("the realm part of UnitName's second return is never read", function()
  with({}, function()
    local f = RD().Collect()
    vm.assert_eq(f.surname, "Brightwood")
  end, { UnitName = function() return "Thalion Brightwood", "Evilrealm" end })
end)

vm.test("long and accented names are cut at character boundaries, never in the middle", function()
  with({ name = "Éloïse " .. string.rep("ü", 20), zone = string.rep("ö", 30) }, function()
    local f = RD().Collect()
    vm.assert_true(#f.surname <= 24 and #f.zone <= 32)
    vm.assert_true(RD().IsUtf8(f.surname) and RD().IsUtf8(f.zone))
    vm.assert_eq(f.firstName, "Éloïse")
    local text = RD().BuildCode()
    vm.assert_true(text and RD().FromText(text))
  end)
end)

vm.test("missing region becomes 0 and is listed", function()
  with({}, function()
    local f, missing = RD().Collect()
    vm.assert_eq(f.region, 0)
    local found = false
    for _, m in ipairs(missing) do if m == "region=?" then found = true end end
    vm.assert_true(found)
  end, { GetCurrentRegion = function() return nil end })
end)

vm.test("zone falls back to GetZoneText and then to an empty string", function()
  with({}, function() vm.assert_eq(RD().Collect().zone, "Elwynn") end,
    { GetRealZoneText = function() return "" end, GetZoneText = function() return "Elwynn" end })
  with({ zone = false }, function() vm.assert_eq(RD().Collect().zone, "") end)
end)

vm.test("events only mark the code stale: no rebuild, no counter change, no timers", function()
  RD().BuildCode()
  vm.assert_true(not RD().stale)
  local counter = RD().counter
  vm.spy("C_Timer.After")
  for _, e in ipairs({ "PLAYER_LEVEL_UP", "ZONE_CHANGED_NEW_AREA", "PLAYER_ENTERING_WORLD", "PLAYER_EQUIPMENT_CHANGED",
    "PLAYER_SPECIALIZATION_CHANGED" }) do
    RD().stale = false
    vm.fire(e)
    vm.assert_true(RD().stale, e .. " marks the code stale")
  end
  vm.assert_eq(RD().counter, counter)
  vm.assert_eq(vm.count("C_Timer.After"), 0)
  vm.assert_eq(vm.count("C_Timer.NewTicker"), 0)
end)

vm.test("item level fallback: without GetAverageItemLevel the slot sum is taken on the equipment event", function()
  local levels = { [1] = 100, [2] = 100, [3] = 100, [5] = 100, [6] = 100, [7] = 100, [8] = 100, [9] = 100, [10] = 100,
    [11] = 100, [12] = 100, [13] = 100, [14] = 100, [15] = 100, [16] = 200 }
  with({}, function()
    vm.fire("PLAYER_EQUIPMENT_CHANGED")
    -- 14 slots at 100, main hand 200, the empty off-hand mirrors it (200), no ranged slot: sum 1800 over 17 slots
    vm.assert_eq(RD().fallbackItemLevel, math.floor((14 * 100 + 200 + 200) / 17 + 0.5))
    vm.assert_eq(RD().Collect().itemLevel, RD().fallbackItemLevel)
  end, {
    GetAverageItemLevel = false,
    GetInventoryItemLink = function(_, slot) return levels[slot] and ("item:" .. slot) or nil end,
    C_Item = { GetDetailedItemLevelInfo = function(link) return levels[tonumber(link:match("%d+"))] end },
  })
  RD().fallbackItemLevel = nil
end)
