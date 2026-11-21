-- The window: the code as copyable text, the same data as a QR code, what the code says, its age and a Refresh button.
-- Plain frames and SetColorTexture fills only. The QR is drawn with one texture per dark module, whole-pixel modules and
-- a four-module quiet zone. Closes with Escape.
local ADDON, NS = ...
local RD = NS.RD
local Window = {}
RD.Window = Window

local WHITE = "Interface\\Buttons\\WHITE8X8"
local QUIET = 4
local QR_AREA = 230 -- the QR (with quiet zone) is drawn inside this many UI units at most
local ROLES = { [0] = "unknown", "tank", "healer", "damage" }

local function box(f, bg, border)
  f:SetBackdrop({ bgFile = WHITE, edgeFile = WHITE, edgeSize = 1, insets = { left = 1, right = 1, top = 1, bottom = 1 } })
  f:SetBackdropColor(bg[1], bg[2], bg[3], 0.97)
  f:SetBackdropBorderColor(border[1], border[2], border[3], 1)
end

local function label(parent, size, r, g, b)
  local fs = parent:CreateFontString(nil, "OVERLAY", "GameFontHighlight")
  if fs.SetFont then fs:SetFont("Fonts\\FRIZQT__.TTF", size or 12, "") end
  fs:SetTextColor(r or 0.9, g or 0.9, b or 0.92)
  fs:SetJustifyH("LEFT")
  return fs
end

local frame = CreateFrame("Frame", "ForeverTogetherFrame", UIParent, "BackdropTemplate")
Window.frame = frame
frame:SetSize(470, 470)
frame:SetPoint("CENTER", 0, 0)
frame:SetFrameStrata("DIALOG")
frame:SetMovable(true)
frame:EnableMouse(true)
frame:RegisterForDrag("LeftButton")
frame:SetScript("OnDragStart", function(self) self:StartMoving() end)
frame:SetScript("OnDragStop", function(self) self:StopMovingOrSizing() end)
box(frame, { 0.08, 0.08, 0.1 }, { 0.8, 0.65, 0.2 })
frame:Hide()
tinsert(UISpecialFrames, "ForeverTogetherFrame")

local title = label(frame, 16, 0.95, 0.8, 0.3)
title:SetPoint("TOPLEFT", 16, -14)
title:SetText("Forever Together: your character code")

local close = CreateFrame("Button", "ForeverTogetherClose", frame, "BackdropTemplate")
close:SetSize(22, 22)
close:SetPoint("TOPRIGHT", -8, -8)
box(close, { 0.15, 0.15, 0.18 }, { 0.4, 0.4, 0.45 })
close.label = label(close, 12)
close.label:SetPoint("CENTER", 0, 0)
close.label:SetText("X")
close:SetScript("OnClick", function() frame:Hide() end)
frame.close = close

local help = label(frame, 12, 0.75, 0.75, 0.8)
help:SetPoint("TOPLEFT", 16, -42)
help:SetWidth(438)
help:SetWordWrap(true)
help:SetText("Click the text and press Ctrl+C, then paste it into the Forever Together app. Or scan the QR code with your phone.")

local edit = CreateFrame("EditBox", "ForeverTogetherCodeBox", frame, "BackdropTemplate")
Window.edit = edit
edit:SetSize(438, 26)
edit:SetPoint("TOPLEFT", 16, -82)
edit:SetAutoFocus(false)
edit:SetFont("Fonts\\ARIALN.TTF", 12, "")
edit:SetTextInsets(8, 8, 0, 0)
edit:SetMaxLetters(0)
box(edit, { 0.02, 0.02, 0.03 }, { 0.4, 0.4, 0.45 })
edit:SetScript("OnEditFocusGained", function(self) self:HighlightText() end)
edit:SetScript("OnEscapePressed", function(self) self:ClearFocus() end)
edit:SetScript("OnTextChanged", function(self, user)
  -- the box is for copying only: any typing is put back
  if user and Window.shownText and self:GetText() ~= Window.shownText then
    self:SetText(Window.shownText)
    self:HighlightText()
  end
end)

local qrHolder = CreateFrame("Frame", "ForeverTogetherQr", frame)
Window.qrHolder = qrHolder
qrHolder:SetPoint("TOPLEFT", 16, -122)
qrHolder:SetSize(QR_AREA, QR_AREA)
local qrBackground = qrHolder:CreateTexture(nil, "BACKGROUND")
qrBackground:SetPoint("TOPLEFT", qrHolder, "TOPLEFT", 0, 0)
qrBackground:SetColorTexture(1, 1, 1, 1)
Window.qrBackground = qrBackground
Window.qrTextures = {} -- one per dark module; extra ones are hidden, not destroyed

local qrNote = label(frame, 12, 0.9, 0.6, 0.4)
qrNote:SetPoint("TOPLEFT", qrHolder, "TOPLEFT", 0, 0)
qrNote:SetWidth(QR_AREA)
qrNote:SetWordWrap(true)
Window.qrNote = qrNote

local info = label(frame, 12)
info:SetPoint("TOPLEFT", 260, -122)
info:SetWidth(200)
info:SetWordWrap(true)
info:SetJustifyV("TOP")
Window.info = info

local age = label(frame, 12, 0.75, 0.75, 0.8)
age:SetPoint("TOPLEFT", 16, -366)
age:SetWidth(438)
Window.age = age

local diag = label(frame, 12, 0.9, 0.6, 0.4)
diag:SetPoint("TOPLEFT", 16, -386)
diag:SetWidth(438)
diag:SetWordWrap(true)
Window.diag = diag

local refresh = CreateFrame("Button", "ForeverTogetherRefresh", frame, "BackdropTemplate")
refresh:SetSize(110, 26)
refresh:SetPoint("BOTTOMLEFT", 16, 14)
box(refresh, { 0.2, 0.17, 0.05 }, { 0.8, 0.65, 0.2 })
refresh.label = label(refresh, 12)
refresh.label:SetPoint("CENTER", 0, 0)
refresh.label:SetText("Refresh")
refresh:SetScript("OnClick", function() Window.Refresh() end)
frame.refresh = refresh

local function ageText()
  local last = RD.last
  if not last then return "No code yet." end
  local minutes = math.max(0, math.floor(((GetServerTime and GetServerTime() or last.created) - last.created) / 60))
  local text = minutes == 0 and "Code made just now." or ("Code made " .. minutes .. (minutes == 1 and " minute" or " minutes") .. " ago.")
  if RD.stale then text = text .. " Your character may have changed: press Refresh." end
  return text
end

local function describe(f)
  local className = UnitClass and UnitClass("player") or "class " .. f.class
  local lines = {
    f.firstName .. " " .. f.surname,
    "Level " .. f.level .. " " .. tostring(className),
    "Faction: " .. f.faction,
    "Role: " .. ROLES[f.role],
    "Item level: " .. (f.itemLevel > 0 and f.itemLevel or "unknown"),
    "Realm: " .. RD.RealmName(f.realm),
    "Region: " .. RD.RegionName(f.region),
    "Zone: " .. (f.zone ~= "" and f.zone or "unknown"),
  }
  return table.concat(lines, "\n")
end

-- Draw the QR for `text`; returns true when drawn.
function Window.DrawQr(text)
  local qr, err = RD.Qr.Encode(text)
  for _, t in ipairs(Window.qrTextures) do t:Hide() end
  Window.qrDark = 0
  if not qr then
    qrBackground:Hide()
    qrNote:SetText(err == "TooLong" and "QR unavailable: the text is too long for a QR code. Use the text above." or "QR unavailable.")
    qrNote:Show()
    return false
  end
  qrNote:Hide()
  local total = qr.size + 2 * QUIET
  local scale = qrHolder.GetEffectiveScale and qrHolder:GetEffectiveScale() or 1
  if type(scale) ~= "number" or scale <= 0 then scale = 1 end
  -- whole screen pixels per module: the biggest that fits the area
  local px = math.max(1, math.floor(QR_AREA * scale / total))
  local module = px / scale
  Window.modulePixels, Window.moduleSize = px, module
  qrBackground:SetSize(total * module, total * module)
  qrBackground:Show()
  local used = 0
  for r = 1, qr.size do
    local row = qr.modules[r]
    for c = 1, qr.size do
      if row[c] then
        used = used + 1
        local t = Window.qrTextures[used]
        if not t then
          t = qrHolder:CreateTexture(nil, "ARTWORK")
          t:SetColorTexture(0, 0, 0, 1)
          Window.qrTextures[used] = t
        end
        t:SetSize(module, module)
        t:ClearAllPoints()
        t:SetPoint("TOPLEFT", qrHolder, "TOPLEFT", (QUIET + c - 1) * module, -(QUIET + r - 1) * module)
        t:Show()
      end
    end
  end
  Window.qrDark = used
  Window.qr = qr
  return true
end

function Window.Show(text, fields, missing, reason)
  Window.shownText = text
  edit:SetText(text or "")
  if text then
    Window.DrawQr(text)
    info:SetText(describe(fields))
  else
    for _, t in ipairs(Window.qrTextures) do t:Hide() end
    qrBackground:Hide()
    qrNote:SetText("No code.")
    qrNote:Show()
    info:SetText(reason or "")
  end
  age:SetText(ageText())
  local parts = {}
  for _, m in ipairs(missing or {}) do parts[#parts + 1] = m end
  diag:SetText(#parts > 0 and ("Not read from the game: " .. table.concat(parts, ", ")) or "")
end

-- Build a new code and show it.
function Window.Refresh()
  local text, a, b, c = RD.BuildCode()
  if text then Window.Show(text, a, c) else Window.Show(nil, nil, b, a) end
  return text
end

function Window.Open()
  frame:Show()
  if RD.stale or not RD.last then Window.Refresh() else Window.Show(RD.last.text, RD.last.fields, RD.last.missing) end
end

function Window.Toggle()
  if frame:IsShown() then frame:Hide() else Window.Open() end
end

-- An event while the window is open only updates the age line (no rebuild: the counter would move on every zone change).
RD.OnStale = function()
  if frame:IsShown() then age:SetText(ageText()) end
end
