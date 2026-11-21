-- QR encoder against matrices made by an independent encoder (segno), stored as rows of 0 and 1 in
-- docs/rdf-code-vectors.json. Qr.Encode returns booleans; they are converted to 0/1 for the comparison.
local Qr = ForeverTogether.RD.Qr
local V = vm.read_json("docs/rdf-code-vectors.json")

local function rows(qr)
  local out = {}
  for r = 1, qr.size do
    local line = {}
    for c = 1, qr.size do line[c] = qr.modules[r][c] and "1" or "0" end
    out[r] = table.concat(line)
  end
  return out
end

vm.test("every QR vector matches the independent encoder module for module", function()
  for _, c in ipairs(V.qr) do
    local qr, err = Qr.Encode(c.text)
    vm.assert_true(qr, c.name .. ": " .. tostring(err))
    vm.assert_eq(qr.version, c.version, c.name .. " version")
    vm.assert_eq(qr.size, c.size, c.name .. " size")
    local got = rows(qr)
    for r = 1, c.size do
      if got[r] ~= c.rows[r] then
        local col
        for k = 1, c.size do if got[r]:sub(k, k) ~= c.rows[r]:sub(k, k) then col = k break end end
        vm.assert_true(false, string.format("%s: first difference at row %d column %d (mask here %d, reference %d)", c.name, r, col, qr.mask, c.mask))
      end
    end
    vm.assert_eq(qr.mask, c.mask, c.name .. " mask")
  end
end)

vm.test("capacity: each version holds exactly the characters of the capacity table, one more needs the next", function()
  for v = 1, 10 do
    local cap = V.qr_capacity_alnum_m[tostring(v)]
    local at = Qr.Encode(string.rep("A", cap))
    vm.assert_eq(at.version, v, "version for " .. cap .. " characters")
    if v < 10 then
      local over = Qr.Encode(string.rep("A", cap + 1))
      vm.assert_eq(over.version, v + 1, "version for " .. (cap + 1) .. " characters")
    end
  end
end)

vm.test("text that cannot be encoded is refused", function()
  for _, c in ipairs(V.qr_errors) do
    local qr, err = Qr.Encode(c.text)
    vm.assert_true(qr == nil, c.name)
    vm.assert_eq(err, c.error, c.name)
  end
end)

vm.test("a typical character code fits version 4 and the largest possible one version 8", function()
  for _, c in ipairs(V.records) do
    if c.name == "typical record" then vm.assert_eq(Qr.Encode(c.text).version, 4) end
    if c.name == "maximum string lengths" then vm.assert_eq(Qr.Encode(c.text).version, 8) end
  end
end)

vm.test("structure: finder patterns, timing pattern and the dark module are in place", function()
  local qr = Qr.Encode("HELLO WORLD")
  local m, n = qr.modules, qr.size
  for _, o in ipairs({ { 1, 1 }, { 1, n - 6 }, { n - 6, 1 } }) do
    for r = 0, 6 do
      for c = 0, 6 do
        local d = math.max(math.abs(r - 3), math.abs(c - 3))
        vm.assert_eq(m[o[1] + r][o[2] + c], d ~= 2, "finder at " .. o[1] .. "," .. o[2])
      end
    end
  end
  for i = 9, n - 8 do
    vm.assert_eq(m[7][i], i % 2 == 1)
    vm.assert_eq(m[i][7], i % 2 == 1)
  end
  vm.assert_eq(m[n - 7][9], true) -- the always-dark module
end)
