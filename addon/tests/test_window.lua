-- The window: opens with /wowft, shows the code and the QR, closes with Escape. The headless VM cannot show how it looks:
-- the layout and the QR on screen are manual checks.
local function RD() return ForeverTogether.RD end
local function W() return RD().Window end

local function shownTextures()
  local n = 0
  for _, t in ipairs(W().qrTextures) do if t:IsShown() then n = n + 1 end end
  return n
end
local function darkModules(qr)
  local n = 0
  for r = 1, qr.size do for c = 1, qr.size do if qr.modules[r][c] then n = n + 1 end end end
  return n
end

vm.test("/wowft opens and closes the window; Escape and the X button close it", function()
  W().frame:Hide()
  vm.slash("/wowft")
  vm.assert_true(W().frame:IsShown())
  vm.slash("/wowft")
  vm.assert_true(not W().frame:IsShown())
  vm.slash("/wowft")
  W().frame.close:Click()
  vm.assert_true(not W().frame:IsShown())
  local escapes = false
  for _, n in ipairs(UISpecialFrames) do if n == "ForeverTogetherFrame" then escapes = true end end
  vm.assert_true(escapes, "Escape closes the window")
end)

vm.test("the code is in a single-line edit box that selects all on focus and cannot be edited", function()
  vm.slash("/wowft")
  local text = RD().last.text
  vm.assert_eq(W().edit:GetText(), text)
  vm.assert_true(text:sub(1, 5) == "WFT1:")
  vm.assert_true(RD().FromText(W().edit:GetText()), "the shown text decodes")
  local highlighted = false
  W().edit.HighlightText = function() highlighted = true end
  vm.fire_script(W().edit, "OnEditFocusGained")
  vm.assert_true(highlighted, "all text is selected when the box is focused")
  W().edit:SetText("tampered")
  vm.fire_script(W().edit, "OnTextChanged", true)
  vm.assert_eq(W().edit:GetText(), text, "typing is put back")
  vm.fire_script(W().edit, "OnEscapePressed")
  W().frame:Hide()
end)

vm.test("the number of shown QR textures equals the number of dark modules, quiet zone excluded", function()
  vm.slash("/wowft")
  local qr = W().qr
  vm.assert_true(qr and qr.version >= 3)
  vm.assert_eq(shownTextures(), darkModules(qr))
  vm.assert_eq(W().qrDark, darkModules(qr))
  -- no texture sits inside the four-module quiet zone, and every one is a whole module in size
  local module = W().moduleSize
  local scale = 1
  vm.assert_eq(W().modulePixels % 1, 0, "whole pixels per module")
  for _, t in ipairs(W().qrTextures) do
    if t:IsShown() then
      local p = t._points[#t._points]
      vm.assert_true(p[4] >= 4 * module - 1e-9 and -p[5] >= 4 * module - 1e-9, "dark module inside the quiet zone")
      vm.assert_true(p[4] <= (4 + qr.size - 1) * module + 1e-9 and -p[5] <= (4 + qr.size - 1) * module + 1e-9)
    end
  end
  W().frame:Hide()
end)

vm.test("drawing a smaller QR hides the textures of the larger one; text that does not fit shows the text only", function()
  W().DrawQr(string.rep("A", 200))
  local big = shownTextures()
  vm.assert_true(big > 0)
  vm.assert_true(W().DrawQr("HELLO WORLD"))
  vm.assert_eq(shownTextures(), darkModules(W().qr))
  vm.assert_true(shownTextures() < big)
  vm.assert_true(not W().DrawQr(string.rep("A", 400)))
  vm.assert_eq(shownTextures(), 0)
  vm.assert_true(W().qrNote:IsShown() and W().qrNote:GetText():find("QR unavailable"))
end)

vm.test("Refresh builds a new code with a higher counter", function()
  vm.slash("/wowft")
  local before = RD().last.fields.counter
  local first = RD().last.text
  W().frame.refresh:Click()
  vm.assert_eq(RD().last.fields.counter, (before + 1) % 65536)
  vm.assert_true(RD().last.text ~= first)
  vm.assert_eq(W().edit:GetText(), RD().last.text)
  vm.assert_eq(shownTextures(), darkModules(W().qr))
  W().frame:Hide()
end)

vm.test("the window shows what the code says, without a dungeon, and the code's age", function()
  RD().stale = true
  vm.slash("/wowft")
  local info = W().info:GetText()
  vm.assert_true(info:find("Thalion Brightwood", 1, true))
  vm.assert_true(info:find("Level 24", 1, true))
  vm.assert_true(info:find("tank", 1, true) and info:find("alliance", 1, true) and info:find("412", 1, true))
  vm.assert_true(info:find("Westfall", 1, true))
  vm.assert_true(not info:lower():find("dungeon"))
  vm.assert_true(W().age:GetText():find("just now"))
  vm.advance(150)
  W().frame:Hide(); vm.slash("/wowft") -- stays as it was: not stale, so no rebuild
  vm.assert_true(W().age:GetText():find("2 minutes"), W().age:GetText())
  vm.fire("PLAYER_LEVEL_UP")
  vm.assert_true(W().age:GetText():find("Refresh"), "a changed character is flagged")
  W().frame:Hide()
end)

vm.test("a character the code cannot be built for shows the reason, not a code", function()
  local player = vm.world.player
  local saved = player.faction
  player.faction = "Neutral"
  RD().stale = true
  W().frame:Hide()
  vm.slash("/wowft")
  vm.assert_eq(W().edit:GetText(), "")
  vm.assert_true(W().info:GetText():find("faction"))
  vm.assert_eq(shownTextures(), 0)
  player.faction = saved
  W().frame:Hide()
end)

vm.test("/wowft code prints the code and /wowft help lists the commands", function()
  RD().stale = true
  vm.clear_chat()
  vm.slash("/wowft code")
  vm.assert_true(vm.find_printed("WFT1:"))
  vm.assert_true(RD().FromText(RD().last.text))
  vm.clear_chat()
  vm.slash("/wowft help")
  vm.assert_true(vm.find_printed("/wowft code"))
  vm.assert_true(not W().frame:IsShown())
end)
