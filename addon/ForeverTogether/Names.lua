-- Display names for the region number and the realm (game-mode record ID). Display only: the code and the topic always carry
-- the raw values. Verified on a live client 2026-11-21: region 3 = EU; realm 14 = PvP, 15 = PvE, 16 = RP-PvE, 18 = RP-PvP.
-- Assumed (standard GetCurrentRegion numbering, not yet seen): 1 US, 2 KR, 4 TW, 5 CN. The RP-PvE and Hardcore IDs are
-- unknown and deliberately absent: they show as "realm <id>" until a live capture names them. Same tables in the app (code.rs).
local ADDON, NS = ...
NS.RD = NS.RD or {}
local RD = NS.RD

RD.REGION_NAMES = { [1] = "US", [2] = "KR", [3] = "EU", [4] = "TW", [5] = "CN" }
RD.REALM_NAMES = { ["14"] = "PvP", ["15"] = "PvE", ["16"] = "RP-PvE", ["18"] = "RP-PvP" }

function RD.RegionName(region)
  if region == 0 then return "unknown" end
  return RD.REGION_NAMES[region] or ("region " .. tostring(region))
end

function RD.RealmName(realm)
  return RD.REALM_NAMES[realm] or ("realm " .. tostring(realm))
end
