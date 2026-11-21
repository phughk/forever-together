-- QR encoder: alphanumeric mode, error correction level M, versions 1 to 10, all eight masks with the ISO penalty rules.
-- Pure Lua 5.1. Qr.Encode(text) -> { size = n, version = v, mask = m, modules = { [row] = { [col] = bool } } } (1-based rows
-- and columns), or nil, "BadText" for text outside the alphanumeric alphabet (or empty) and nil, "TooLong" when it does
-- not fit version 10. Checked against matrices from an independent encoder in docs/rdf-code-vectors.json.
local ADDON, NS = ...
NS.RD = NS.RD or {}
local RD = NS.RD
local Qr = {}
RD.Qr = Qr

local floor = math.floor
local ALPHABET = "0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ $%*+-./:"
local INDEX = {}
for i = 1, #ALPHABET do INDEX[ALPHABET:sub(i, i)] = i - 1 end

-- Level M. total codewords, error codewords per block, blocks as { count, data codewords per block } groups.
local VERSIONS = {
  { ec = 10, groups = { { 1, 16 } } },
  { ec = 16, groups = { { 1, 28 } } },
  { ec = 26, groups = { { 1, 44 } } },
  { ec = 18, groups = { { 2, 32 } } },
  { ec = 24, groups = { { 2, 43 } } },
  { ec = 16, groups = { { 4, 27 } } },
  { ec = 18, groups = { { 4, 31 } } },
  { ec = 22, groups = { { 2, 38 }, { 2, 39 } } },
  { ec = 22, groups = { { 3, 36 }, { 2, 37 } } },
  { ec = 26, groups = { { 4, 43 }, { 1, 44 } } },
}
local ALIGN = { [2] = { 6, 18 }, [3] = { 6, 22 }, [4] = { 6, 26 }, [5] = { 6, 30 }, [6] = { 6, 34 }, [7] = { 6, 22, 38 },
  [8] = { 6, 24, 42 }, [9] = { 6, 26, 46 }, [10] = { 6, 28, 50 } }
local REMAINDER_BITS = { 0, 7, 7, 7, 7, 7, 0, 0, 0, 0 }

local function bxor(a, b)
  local r, bit = 0, 1
  while a > 0 or b > 0 do
    local x, y = a % 2, b % 2
    if x ~= y then r = r + bit end
    a, b, bit = (a - x) / 2, (b - y) / 2, bit * 2
  end
  return r
end
local function bitOf(n, i) return floor(n / 2 ^ i) % 2 end

-- GF(256), polynomial 0x11D
local EXP, LOG = {}, {}
do
  local x = 1
  for i = 0, 254 do
    EXP[i], LOG[x] = x, i
    x = x * 2
    if x >= 256 then x = bxor(x, 0x11D) end
  end
  for i = 255, 509 do EXP[i] = EXP[i - 255] end
end
local function gmul(a, b)
  if a == 0 or b == 0 then return 0 end
  return EXP[LOG[a] + LOG[b]]
end

local generators = {}
local function generator(n) -- coefficients of x^(n-1) .. x^0 (the leading 1 is left out)
  if generators[n] then return generators[n] end
  local g = { 1 }
  for i = 0, n - 1 do
    local nxt = {}
    for k = 1, #g + 1 do nxt[k] = 0 end
    for k = 1, #g do
      nxt[k] = bxor(nxt[k], g[k])
      nxt[k + 1] = bxor(nxt[k + 1], gmul(g[k], EXP[i]))
    end
    g = nxt
  end
  local out = {}
  for k = 2, #g do out[k - 1] = g[k] end
  generators[n] = out
  return out
end

local function reedSolomon(data, n)
  local gen = generator(n)
  local rem = {}
  for i = 1, n do rem[i] = 0 end
  for _, b in ipairs(data) do
    local factor = bxor(b, rem[1])
    table.remove(rem, 1)
    rem[n] = 0
    for i = 1, n do rem[i] = bxor(rem[i], gmul(gen[i], factor)) end
  end
  return rem
end

local function dataCapacity(v)
  local info, total = VERSIONS[v], 0
  for _, g in ipairs(info.groups) do total = total + g[1] * g[2] end
  return total
end

local function chooseVersion(n)
  local bitsNeeded = 11 * floor(n / 2) + 6 * (n % 2)
  for v = 1, 10 do
    local countBits = v <= 9 and 9 or 11
    if 4 + countBits + bitsNeeded <= dataCapacity(v) * 8 then return v end
  end
end

local function dataCodewords(text, v)
  local bits = {}
  local function put(value, count)
    for i = count - 1, 0, -1 do bits[#bits + 1] = bitOf(value, i) end
  end
  put(2, 4) -- alphanumeric mode
  put(#text, v <= 9 and 9 or 11)
  for i = 1, #text - 1, 2 do put(INDEX[text:sub(i, i)] * 45 + INDEX[text:sub(i + 1, i + 1)], 11) end
  if #text % 2 == 1 then put(INDEX[text:sub(#text)], 6) end
  local capBits = dataCapacity(v) * 8
  for _ = 1, math.min(4, capBits - #bits) do bits[#bits + 1] = 0 end
  while #bits % 8 ~= 0 do bits[#bits + 1] = 0 end
  local words = {}
  for i = 1, #bits, 8 do
    local w = 0
    for k = 0, 7 do w = w * 2 + bits[i + k] end
    words[#words + 1] = w
  end
  local pad = { 0xEC, 0x11 }
  local k = 0
  while #words < dataCapacity(v) do words[#words + 1] = pad[k % 2 + 1]; k = k + 1 end
  return words
end

-- split into blocks, add error correction, interleave, return the bit list including remainder bits
local function finalBits(words, v)
  local info = VERSIONS[v]
  local blocks, pos = {}, 1
  for _, g in ipairs(info.groups) do
    for _ = 1, g[1] do
      local d = {}
      for i = 1, g[2] do d[i] = words[pos]; pos = pos + 1 end
      blocks[#blocks + 1] = { data = d, ec = reedSolomon(d, info.ec) }
    end
  end
  local out, maxLen = {}, 0
  for _, b in ipairs(blocks) do if #b.data > maxLen then maxLen = #b.data end end
  for i = 1, maxLen do
    for _, b in ipairs(blocks) do if b.data[i] then out[#out + 1] = b.data[i] end end
  end
  for i = 1, info.ec do
    for _, b in ipairs(blocks) do out[#out + 1] = b.ec[i] end
  end
  local bits = {}
  for _, w in ipairs(out) do
    for i = 7, 0, -1 do bits[#bits + 1] = bitOf(w, i) end
  end
  for _ = 1, REMAINDER_BITS[v] do bits[#bits + 1] = 0 end
  return bits
end

local MASKS = {
  function(i, j) return (i + j) % 2 == 0 end,
  function(i, j) return i % 2 == 0 end,
  function(i, j) return j % 3 == 0 end,
  function(i, j) return (i + j) % 3 == 0 end,
  function(i, j) return (floor(i / 2) + floor(j / 3)) % 2 == 0 end,
  function(i, j) return (i * j) % 2 + (i * j) % 3 == 0 end,
  function(i, j) return ((i * j) % 2 + (i * j) % 3) % 2 == 0 end,
  function(i, j) return ((i + j) % 2 + (i * j) % 3) % 2 == 0 end,
}

-- 0-based rows and columns inside; m[r][c] is 0 or 1
local function newMatrix(size)
  local m, fn = {}, {}
  for r = 0, size - 1 do
    m[r], fn[r] = {}, {}
    for c = 0, size - 1 do m[r][c], fn[r][c] = 0, false end
  end
  return m, fn
end

local function drawFunctionPatterns(m, fn, size, v)
  local function set(r, c, val) m[r][c] = val and 1 or 0; fn[r][c] = true end
  -- finder patterns with separators
  for _, corner in ipairs({ { 0, 0 }, { 0, size - 7 }, { size - 7, 0 } }) do
    for dr = -1, 7 do
      for dc = -1, 7 do
        local r, c = corner[1] + dr, corner[2] + dc
        if r >= 0 and r < size and c >= 0 and c < size then
          local dist = math.max(math.abs(dr - 3), math.abs(dc - 3))
          set(r, c, dist ~= 2 and dist ~= 4)
        end
      end
    end
  end
  -- timing
  for i = 8, size - 9 do set(6, i, i % 2 == 0); set(i, 6, i % 2 == 0) end
  -- alignment
  local pos = ALIGN[v]
  if pos then
    for _, cr in ipairs(pos) do
      for _, cc in ipairs(pos) do
        local overlapsFinder = (cr == 6 and cc == 6) or (cr == 6 and cc == pos[#pos]) or (cr == pos[#pos] and cc == 6)
        if not overlapsFinder then
          for dr = -2, 2 do
            for dc = -2, 2 do set(cr + dr, cc + dc, math.max(math.abs(dr), math.abs(dc)) ~= 1) end
          end
        end
      end
    end
  end
  -- reserve format and version areas (drawn after masking)
  for i = 0, 8 do fn[8][i] = true; fn[i][8] = true end
  for i = 0, 7 do fn[8][size - 1 - i] = true; fn[size - 1 - i][8] = true end
  set(size - 8, 8, true) -- the always-dark module
  if v >= 7 then
    for i = 0, 5 do
      for k = 0, 2 do fn[i][size - 11 + k] = true; fn[size - 11 + k][i] = true end
    end
  end
end

local function placeData(m, fn, size, bits)
  local idx, up = 1, true
  local col = size - 1
  while col > 0 do
    if col == 6 then col = col - 1 end
    for step = 0, size - 1 do
      local r = up and (size - 1 - step) or step
      for k = 0, 1 do
        local c = col - k
        if not fn[r][c] then
          m[r][c] = bits[idx] or 0
          idx = idx + 1
        end
      end
    end
    up = not up
    col = col - 2
  end
end

-- Penalty rules as in the reference encoder used for the vectors: evaluated before the format and version information and the
-- dark module are drawn. N1 runs of five or more (3 + extra), N2 2x2 blocks (3), N3 dark-light-dark-dark-dark-light-dark with light on
-- one side or at the edge (40), N4 dark share (10 per 5 percent away from 50).
local function n3Count(seq, size)
  local count, i = 0, 0
  while i <= size - 7 do
    if seq[i] == 1 and seq[i + 1] == 0 and seq[i + 2] == 1 and seq[i + 3] == 1 and seq[i + 4] == 1 and seq[i + 5] == 0
      and seq[i + 6] == 1 then
      local before, after = true, true
      for k = math.max(i - 4, 0), i - 1 do if seq[k] ~= 0 then before = false end end
      for k = i + 7, math.min(i + 10, size - 1) do if seq[k] ~= 0 then after = false end end
      if i == 0 or i == size - 7 or before or after then count = count + 40; i = i + 7 else i = i + 4 end
    else
      i = i + 1
    end
  end
  return count
end

local function penalty(m, size)
  local n1, n2, n3, dark = 0, 0, 0, 0
  for r = 0, size - 1 do
    local rowRun, colRun = 1, 1
    local col = {}
    for c = 0, size - 1 do
      col[c] = m[c][r]
      dark = dark + m[r][c]
      if c > 0 then
        if m[r][c] == m[r][c - 1] then rowRun = rowRun + 1 else if rowRun >= 5 then n1 = n1 + rowRun - 2 end rowRun = 1 end
        if m[c][r] == m[c - 1][r] then colRun = colRun + 1 else if colRun >= 5 then n1 = n1 + colRun - 2 end colRun = 1 end
        if r > 0 and m[r][c] == m[r][c - 1] and m[r][c] == m[r - 1][c] and m[r][c] == m[r - 1][c - 1] then n2 = n2 + 3 end
      end
    end
    if rowRun >= 5 then n1 = n1 + rowRun - 2 end
    if colRun >= 5 then n1 = n1 + colRun - 2 end
    n3 = n3 + n3Count(m[r], size) + n3Count(col, size)
  end
  local n4 = 10 * floor(math.abs(dark * 100 / (size * size) - 50) / 5)
  return n1 + n2 + n3 + n4
end

local function drawFormat(m, size, mask)
  local data = mask -- error level M is 00, then the mask number
  local rem = data
  for _ = 1, 10 do rem = bxor(rem * 2, floor(rem / 512) * 0x537) end
  local bits = bxor(data * 1024 + rem, 0x5412)
  local function set(r, c, i) m[r][c] = bitOf(bits, i) end
  for i = 0, 5 do set(i, 8, i) end
  set(7, 8, 6); set(8, 8, 7); set(8, 7, 8)
  for i = 9, 14 do set(8, 14 - i, i) end
  for i = 0, 7 do set(8, size - 1 - i, i) end
  for i = 8, 14 do set(size - 15 + i, 8, i) end
  m[size - 8][8] = 1
end

local function drawVersion(m, size, v)
  if v < 7 then return end
  local rem = v
  for _ = 1, 12 do rem = bxor(rem * 2, floor(rem / 2048) * 0x1F25) end
  local bits = v * 4096 + rem
  for i = 0, 17 do
    local a, b = size - 11 + i % 3, floor(i / 3)
    m[b][a] = bitOf(bits, i)
    m[a][b] = bitOf(bits, i)
  end
end

function Qr.Encode(text)
  if type(text) ~= "string" or #text == 0 then return nil, "BadText" end
  for i = 1, #text do if not INDEX[text:sub(i, i)] then return nil, "BadText" end end
  local v = chooseVersion(#text)
  if not v then return nil, "TooLong" end
  local size = 17 + 4 * v
  local bits = finalBits(dataCodewords(text, v), v)
  local base, fn = newMatrix(size)
  drawFunctionPatterns(base, fn, size, v)
  placeData(base, fn, size, bits)
  local bestScore, bestMask, best
  for mask = 0, 7 do
    local m = {}
    for r = 0, size - 1 do
      m[r] = {}
      for c = 0, size - 1 do
        local val = base[r][c]
        if not fn[r][c] and MASKS[mask + 1](r, c) then val = 1 - val end
        m[r][c] = val
      end
    end
    m[size - 8][8] = 0 -- the reference encoder scores the matrix before the dark module is drawn
    local score = penalty(m, size)
    m[size - 8][8] = 1
    if not bestScore or score < bestScore then bestScore, bestMask, best = score, mask, m end
  end
  drawFormat(best, size, bestMask)
  drawVersion(best, size, v)
  local modules = {}
  for r = 1, size do
    modules[r] = {}
    for c = 1, size do modules[r][c] = best[r - 1][c - 1] == 1 end
  end
  return { size = size, version = v, mask = bestMask, modules = modules }
end

-- Longest text that fits version 10, for callers that want to check before encoding.
Qr.MAX_CHARS = 311
