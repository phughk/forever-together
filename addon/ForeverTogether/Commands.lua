-- /wowft          toggle the window
-- /wowft code     print the code in chat (a fresh one when the last is stale)
-- /wowft help     list the commands
local ADDON, NS = ...
local RD = NS.RD

SLASH_FOREVERTOGETHER1 = "/wowft"

local PREFIX = "|cff33ff99Forever Together|r: "

SlashCmdList["FOREVERTOGETHER"] = function(msg)
  msg = strtrim(msg or ""):lower()
  if msg == "" then
    RD.Window.Toggle()
  elseif msg == "code" then
    local text, a, b
    if RD.last and not RD.stale then text = RD.last.text else text, a, b = RD.BuildCode() end
    if text then
      print(PREFIX .. text)
    else
      local why = a or "cannot build the code"
      if type(b) == "table" and #b > 0 then why = why .. " (" .. table.concat(b, ", ") .. ")" end
      print(PREFIX .. why)
    end
  else
    print(PREFIX .. "/wowft opens or closes the window, /wowft code prints the code in chat, /wowft help shows this.")
  end
end
