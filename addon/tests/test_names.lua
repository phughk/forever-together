-- Region and realm display names (Names.lua) and how the window shows them. Display only: the code carries the raw values.
local V = vm.read_json("docs/rdf-code-vectors.json")
local function RD() return ForeverTogether.RD end
local function W() return RD().Window end

vm.test("region numbers map to names, 0 is unknown, others show their number", function()
  vm.assert_eq(RD().RegionName(3), "EU")
  vm.assert_eq(RD().RegionName(1), "US")
  vm.assert_eq(RD().RegionName(0), "unknown")
  vm.assert_eq(RD().RegionName(9), "region 9")
end)

vm.test("realm IDs map to rulesets, others show 'realm <id>' (Hardcore is not released, so not known)", function()
  vm.assert_eq(RD().RealmName("14"), "PvP")
  vm.assert_eq(RD().RealmName("15"), "PvE")
  vm.assert_eq(RD().RealmName("16"), "RP-PvE")
  vm.assert_eq(RD().RealmName("18"), "RP-PvP")
  vm.assert_eq(RD().RealmName("99"), "realm 99")
  vm.assert_eq(RD().RealmName("mankrik"), "realm mankrik")
end)

vm.test("the window names the realm and region of every synthetic realm vector", function()
  local want = { ["14"] = "Realm: PvP", ["15"] = "Realm: PvE", ["18"] = "Realm: RP-PvP", ["27"] = "Realm: realm 27" }
  local seen = 0
  for _, c in ipairs(V.records) do
    if c.name:find("^synthetic:") then
      local f = RD().FromText(c.text)
      vm.assert_true(f, c.name)
      W().Show(c.text, f, {})
      local info = W().info:GetText()
      vm.assert_true(info:find(want[f.realm], 1, true), c.name .. ": " .. info)
      vm.assert_true(info:find("Region: EU", 1, true), c.name .. ": " .. info)
      seen = seen + 1
    end
  end
  vm.assert_eq(seen, 4)
end)
