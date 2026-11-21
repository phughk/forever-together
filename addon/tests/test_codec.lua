-- Codec against docs/rdf-code-vectors.json (made with an independent Python codec). Run from the repository root.
local RD = ForeverTogether.RD
local V = vm.read_json("docs/rdf-code-vectors.json")

local function fromHex(h)
  return (h:gsub("%x%x", function(x) return string.char(tonumber(x, 16)) end))
end
local function toHex(s)
  return (s:gsub(".", function(c) return string.format("%02x", c:byte()) end))
end
local function sameFields(want, got, label)
  for k, v in pairs(want) do vm.assert_eq(got[k], v, label .. ": field " .. k) end
  for k in pairs(got) do vm.assert_true(want[k] ~= nil, label .. ": unexpected field " .. k) end
end

vm.test("vectors file has every required case", function()
  local names = {}
  for _, r in ipairs(V.records) do names[r.name] = true end
  for _, want in ipairs({ "minimal record", "maximum string lengths", "accented names", "item level unknown", "role tank",
    "role healer", "role damage", "role unknown", "faction alliance", "faction horde", "counter 65535", "internal space in text" }) do
    vm.assert_true(names[want], "missing record vector: " .. want)
  end
  local errs = {}
  for _, e in ipairs(V.decode_errors) do errs[e.error] = true end
  for _, want in ipairs({ "BadCheck", "Truncated", "UnknownVersion", "TrailingBytes", "BadText", "InvalidField(level)" }) do
    vm.assert_true(errs[want], "missing error vector: " .. want)
  end
  vm.assert_true(#V.qr >= 5 and #V.base45 >= 8)
end)

vm.test("base45 encodes and decodes every vector (odd and even lengths)", function()
  local odd, even = false, false
  for _, c in ipairs(V.base45) do
    local bytes = fromHex(c.hex)
    vm.assert_eq(RD.Base45Encode(bytes), c.text, c.name .. " encode")
    vm.assert_eq(toHex(RD.Base45Decode(c.text)), c.hex, c.name .. " decode")
    if #bytes % 2 == 1 then odd = true else even = true end
  end
  vm.assert_true(odd and even)
end)

vm.test("base45 refuses bad text", function()
  for _, c in ipairs(V.base45_errors) do
    local out, err = RD.Base45Decode(c.text)
    vm.assert_true(out == nil and err == "BadText", c.name)
  end
end)

vm.test("CRC-16/CCITT-FALSE vectors", function()
  for _, c in ipairs(V.crc16) do vm.assert_eq(RD.Crc16(fromHex(c.hex)), c.crc, c.name) end
end)

vm.test("every record vector encodes to the same bytes and text, and decodes back to its fields", function()
  for _, c in ipairs(V.records) do
    local bytes, err = RD.EncodeRecord(c.fields)
    vm.assert_true(bytes, c.name .. ": " .. tostring(err))
    vm.assert_eq(toHex(bytes), c.hex, c.name .. " bytes")
    vm.assert_eq(RD.ToText(c.fields), c.text, c.name .. " text")
    local back, derr = RD.FromText(c.text)
    vm.assert_true(back, c.name .. " decode: " .. tostring(derr))
    sameFields(c.fields, back, c.name)
    local viaBytes = RD.DecodeRecord(bytes)
    sameFields(c.fields, viaBytes, c.name .. " (bytes)")
  end
end)

vm.test("every decode error vector gives the named error", function()
  for _, c in ipairs(V.decode_errors) do
    local f, err = RD.FromText(c.text)
    vm.assert_true(f == nil, c.name .. ": should not decode")
    vm.assert_eq(err, c.error, c.name)
  end
end)

vm.test("pasted text: outer whitespace and line breaks are stripped, a space inside is kept", function()
  for _, c in ipairs(V.paste) do
    local a = RD.FromText(c.pasted)
    local b = RD.FromText(c.text)
    vm.assert_true(a and b, c.name)
    sameFields(b, a, c.name)
  end
  local internal
  for _, c in ipairs(V.records) do if c.name == "internal space in text" then internal = c end end
  vm.assert_true(internal.text:find(" ", 6, true), "the vector must contain an internal space")
  local stripped = internal.text:gsub(" ", "", 1)
  local f, err = RD.FromText(stripped)
  vm.assert_true(f == nil, "removing the internal space must not decode")
end)

vm.test("the encoder refuses invalid input with InvalidField(name)", function()
  for _, c in ipairs(V.encode_errors) do
    local out, err = RD.EncodeRecord(c.fields)
    vm.assert_true(out == nil, c.name)
    vm.assert_eq(err, c.error, c.name)
  end
end)

vm.test("a text with no record bytes at all and non-string input are BadText or Truncated", function()
  local f, err = RD.FromText(nil)
  vm.assert_true(f == nil and err == "BadText")
  f, err = RD.FromText("")
  vm.assert_true(f == nil and err == "BadText")
end)

vm.test("TruncateUtf8 never cuts inside a character", function()
  vm.assert_eq(RD.TruncateUtf8("abc", 5), "abc")
  vm.assert_eq(RD.TruncateUtf8("abcdef", 3), "abc")
  vm.assert_eq(RD.TruncateUtf8("ééé", 5), "éé") -- 2 bytes each: 5 would cut the third in half
  vm.assert_eq(RD.TruncateUtf8("ééé", 4), "éé")
  vm.assert_eq(RD.TruncateUtf8("é", 1), "")
  vm.assert_eq(RD.TruncateUtf8("a€b", 3), "a") -- € is 3 bytes
  vm.assert_true(RD.IsUtf8(RD.TruncateUtf8("日本語のテキスト", 10)))
end)

vm.test("size: the largest possible record is 194 characters", function()
  local longest
  for _, c in ipairs(V.records) do if c.name == "maximum string lengths" then longest = c end end
  vm.assert_eq(#longest.text, 194)
  vm.assert_eq(#fromHex(longest.hex), V.limits.max_record_bytes)
  local min
  for _, c in ipairs(V.records) do if c.name == "minimal record" then min = c end end
  vm.assert_eq(#fromHex(min.hex), V.limits.min_record_bytes)
end)
