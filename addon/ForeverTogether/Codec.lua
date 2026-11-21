-- Character code: "WFT1:" + base45(record). Layout and errors: docs/rdf-code-spec.md, section 1.
-- Pure Lua 5.1, no game API, so the same file runs in the headless test VM and in the client.
local ADDON, NS = ...
NS.RD = NS.RD or {}
local RD = NS.RD
_G.ForeverTogether = NS -- tests and the other files reach the namespace through this global after a reload

local floor, char, byte, sub, concat = math.floor, string.char, string.byte, string.sub, table.concat

RD.PREFIX = "WFT1:"
RD.VERSION = 1
RD.MIN_RECORD_BYTES = 25 -- 16 fixed + realm, first name, surname (2 each) + zone length (1) + check (2)
RD.MAX_REALM, RD.MAX_NAME, RD.MAX_ZONE = 24, 24, 32

local ALPHABET = "0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ $%*+-./:"
local INDEX = {}
for i = 1, #ALPHABET do INDEX[sub(ALPHABET, i, i)] = i - 1 end
RD.BASE45_ALPHABET = ALPHABET

-- RFC 9285 base45. Input and output of the byte side are Lua strings.
function RD.Base45Encode(bytes)
  local out, n = {}, #bytes
  local i = 1
  while i <= n do
    if i < n then
      local v = byte(bytes, i) * 256 + byte(bytes, i + 1)
      local e = floor(v / 2025); v = v - e * 2025
      local d = floor(v / 45); local c = v - d * 45
      out[#out + 1] = sub(ALPHABET, c + 1, c + 1) .. sub(ALPHABET, d + 1, d + 1) .. sub(ALPHABET, e + 1, e + 1)
      i = i + 2
    else
      local v = byte(bytes, i)
      local d = floor(v / 45); local c = v - d * 45
      out[#out + 1] = sub(ALPHABET, c + 1, c + 1) .. sub(ALPHABET, d + 1, d + 1)
      i = i + 1
    end
  end
  return concat(out)
end

-- Returns the bytes, or nil, "BadText".
function RD.Base45Decode(text)
  local n = #text
  if n % 3 == 1 then return nil, "BadText" end
  local out = {}
  local i = 1
  while i <= n do
    local c = INDEX[sub(text, i, i)]
    local d = INDEX[sub(text, i + 1, i + 1)]
    if not c or not d then return nil, "BadText" end
    if i + 2 <= n then
      local e = INDEX[sub(text, i + 2, i + 2)]
      if not e then return nil, "BadText" end
      local v = c + d * 45 + e * 2025
      if v > 65535 then return nil, "BadText" end
      out[#out + 1] = char(floor(v / 256), v % 256)
      i = i + 3
    else
      local v = c + d * 45
      if v > 255 then return nil, "BadText" end
      out[#out + 1] = char(v)
      i = i + 2
    end
  end
  return concat(out)
end

-- CRC-16/CCITT-FALSE: poly 0x1021, init 0xFFFF, no reflection, no final xor. Check value for "123456789" is 0x29B1.
local crcTable
function RD.Crc16(bytes)
  if not crcTable then
    crcTable = {}
    for i = 0, 255 do
      local crc = i * 256
      for _ = 1, 8 do
        if crc >= 32768 then crc = ((crc * 2) % 65536); crc = RD.Xor16(crc, 0x1021) else crc = crc * 2 end
      end
      crcTable[i] = crc
    end
  end
  local crc = 0xFFFF
  for i = 1, #bytes do
    local idx = RD.Xor16(floor(crc / 256), byte(bytes, i))
    crc = RD.Xor16((crc % 256) * 256, crcTable[idx])
  end
  return crc
end

-- xor of two numbers below 65536 (Lua 5.1 has no bit operators)
function RD.Xor16(a, b)
  local r, bit = 0, 1
  for _ = 1, 16 do
    local x, y = a % 2, b % 2
    if x ~= y then r = r + bit end
    a, b, bit = (a - x) / 2, (b - y) / 2, bit * 2
  end
  return r
end

-- Is this string well-formed UTF-8 (no overlong forms, no surrogates, nothing above U+10FFFF)?
function RD.IsUtf8(s)
  local i, n = 1, #s
  while i <= n do
    local c = byte(s, i)
    local len, min
    if c < 0x80 then len, min = 1, 0
    elseif c >= 0xC2 and c <= 0xDF then len, min = 2, 0x80
    elseif c >= 0xE0 and c <= 0xEF then len, min = 3, 0x800
    elseif c >= 0xF0 and c <= 0xF4 then len, min = 4, 0x10000
    else return false end
    if i + len - 1 > n then return false end
    local cp = len == 1 and c or (len == 2 and c - 0xC0) or (len == 3 and c - 0xE0) or (c - 0xF0)
    for k = 1, len - 1 do
      local cc = byte(s, i + k)
      if cc < 0x80 or cc > 0xBF then return false end
      cp = cp * 64 + (cc - 0x80)
    end
    if cp < min or cp > 0x10FFFF or (cp >= 0xD800 and cp <= 0xDFFF) then return false end
    i = i + len
  end
  return true
end

-- Cut a string to at most max bytes without splitting a UTF-8 character.
function RD.TruncateUtf8(s, max)
  if #s <= max then return s end
  local cut = max
  while cut > 0 do
    local c = byte(s, cut + 1)
    if c < 0x80 or c >= 0xC0 then break end -- the next byte starts a character, so cutting here is clean
    cut = cut - 1
  end
  return sub(s, 1, cut)
end

local function u16(n) return char(floor(n / 256), n % 256) end
local function u32(n) return char(floor(n / 16777216) % 256, floor(n / 65536) % 256, floor(n / 256) % 256, n % 256) end

local function isInt(v, lo, hi) return type(v) == "number" and v == floor(v) and v >= lo and v <= hi end
local function str(s, name, minLen, maxLen)
  if type(s) ~= "string" or #s < minLen or #s > maxLen or not RD.IsUtf8(s) then return nil, "InvalidField(" .. name .. ")" end
  return char(#s) .. s
end

-- fields: role (0-3), secondaryRole (0-3), faction ("alliance" | "horde"), itemLevel (0 = unknown), class, race, level,
-- spec, created, counter, region, realm, firstName, surname, zone. Returns the record bytes, or nil, "InvalidField(name)".
function RD.EncodeRecord(f)
  if not isInt(f.role, 0, 3) then return nil, "InvalidField(role)" end
  if not isInt(f.secondaryRole or 0, 0, 3) then return nil, "InvalidField(secondaryRole)" end
  if f.faction ~= "alliance" and f.faction ~= "horde" then return nil, "InvalidField(faction)" end
  if not isInt(f.itemLevel, 0, 65535) then return nil, "InvalidField(itemLevel)" end
  if not isInt(f.class, 0, 255) then return nil, "InvalidField(class)" end
  if not isInt(f.race, 0, 255) then return nil, "InvalidField(race)" end
  if not isInt(f.level, 1, 255) then return nil, "InvalidField(level)" end
  if not isInt(f.spec, 0, 65535) then return nil, "InvalidField(spec)" end
  if not isInt(f.created, 0, 4294967295) then return nil, "InvalidField(created)" end
  if not isInt(f.counter, 0, 65535) then return nil, "InvalidField(counter)" end
  if not isInt(f.region, 0, 255) then return nil, "InvalidField(region)" end
  local realm, e1 = str(f.realm, "realm", 1, RD.MAX_REALM); if not realm then return nil, e1 end
  local first, e2 = str(f.firstName, "firstName", 1, RD.MAX_NAME); if not first then return nil, e2 end
  local sur, e3 = str(f.surname, "surname", 1, RD.MAX_NAME); if not sur then return nil, e3 end
  local zone, e4 = str(f.zone or "", "zone", 0, RD.MAX_ZONE); if not zone then return nil, e4 end
  local flags = f.role + (f.secondaryRole or 0) * 4 + (f.faction == "horde" and 16 or 0) + (f.itemLevel > 0 and 32 or 0)
  local body = char(RD.VERSION, flags, f.class, f.race, f.level) .. u16(f.itemLevel) .. u16(f.spec) .. u32(f.created)
    .. u16(f.counter) .. char(f.region) .. realm .. first .. sur .. zone
  return body .. u16(RD.Crc16(body))
end

-- Order of checks: version, minimum length, check value, then the fields. So a record from a newer version is reported
-- as UnknownVersion whatever its layout, damage is reported as BadCheck, and Truncated/TrailingBytes/InvalidField only
-- appear for records whose check value is correct (or that are too short to carry one).
-- Returns the fields table, or nil, err.
function RD.DecodeRecord(b)
  if #b < 1 then return nil, "Truncated" end
  if byte(b, 1) ~= RD.VERSION then return nil, "UnknownVersion" end
  if #b < RD.MIN_RECORD_BYTES then return nil, "Truncated" end
  local body = sub(b, 1, #b - 2)
  if RD.Crc16(body) ~= byte(b, #b - 1) * 256 + byte(b, #b) then return nil, "BadCheck" end
  local flags = byte(b, 2)
  if flags >= 64 then return nil, "InvalidField(flags)" end
  local level = byte(b, 5)
  if level == 0 then return nil, "InvalidField(level)" end
  local f = {
    role = flags % 4, secondaryRole = floor(flags / 4) % 4, faction = (floor(flags / 16) % 2 == 1) and "horde" or "alliance",
    class = byte(b, 3), race = byte(b, 4), level = level,
    itemLevel = floor(flags / 32) % 2 == 1 and (byte(b, 6) * 256 + byte(b, 7)) or 0,
    spec = byte(b, 8) * 256 + byte(b, 9),
    created = byte(b, 10) * 16777216 + byte(b, 11) * 65536 + byte(b, 12) * 256 + byte(b, 13),
    counter = byte(b, 14) * 256 + byte(b, 15), region = byte(b, 16),
  }
  local pos, last = 17, #b - 2 -- last byte of the body
  local function readString(name, minLen, maxLen)
    if pos > last then return nil, "Truncated" end
    local len = byte(b, pos)
    if len > maxLen then return nil, "InvalidField(" .. name .. ")" end
    if pos + len > last then return nil, "Truncated" end
    local s = sub(b, pos + 1, pos + len)
    pos = pos + 1 + len
    if len < minLen or not RD.IsUtf8(s) then return nil, "InvalidField(" .. name .. ")" end
    return s
  end
  local err
  f.realm, err = readString("realm", 1, RD.MAX_REALM); if not f.realm then return nil, err end
  f.firstName, err = readString("firstName", 1, RD.MAX_NAME); if not f.firstName then return nil, err end
  f.surname, err = readString("surname", 1, RD.MAX_NAME); if not f.surname then return nil, err end
  f.zone, err = readString("zone", 0, RD.MAX_ZONE); if not f.zone then return nil, err end
  if pos <= last then return nil, "TrailingBytes" end
  return f
end

function RD.ToText(fields)
  local rec, err = RD.EncodeRecord(fields)
  if not rec then return nil, err end
  return RD.PREFIX .. RD.Base45Encode(rec)
end

-- Strips leading and trailing whitespace only: a space inside the text is a base45 character.
function RD.FromText(text)
  if type(text) ~= "string" then return nil, "BadText" end
  text = text:gsub("^%s+", ""):gsub("%s+$", "")
  if sub(text, 1, #RD.PREFIX) ~= RD.PREFIX then return nil, "BadText" end
  local bytes, err = RD.Base45Decode(sub(text, #RD.PREFIX + 1))
  if not bytes then return nil, err end
  return RD.DecodeRecord(bytes)
end
