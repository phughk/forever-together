import json, sys, segno
import segno.encoder as segno_encoder
import qrcode, qrcode.util, qrcode.constants

# segno (1.6.6) appends a spurious all-zero byte when the data bits plus terminator end exactly on a byte boundary
# (write_padding_bits adds 8 zero bits instead of none). That is legal to scan but not the canonical layout, so the
# padding function is patched here and every matrix is cross-checked against a second encoder (python-qrcode) forced to
# the same version and mask.
def _pad_fixed(buff, version, length):
    buff.extend([0] * ((8 - (length % 8)) % 8))
segno_encoder.write_padding_bits = _pad_fixed

# Independent reference for docs/rdf-code-vectors.json: its own base45, CRC-16 and record writer, and the
# `segno` library (not the addon's Qr.lua) for the QR matrices.
ALPHA = "0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ $%*+-./:"
PREFIX = "WFT1:"


def b45enc(b):
    out = ""
    for i in range(0, len(b), 2):
        if i + 1 < len(b):
            n = b[i] * 256 + b[i + 1]
            e, rest = divmod(n, 45 * 45)
            d, c = divmod(rest, 45)
            out += ALPHA[c] + ALPHA[d] + ALPHA[e]
        else:
            d, c = divmod(b[i], 45)
            out += ALPHA[c] + ALPHA[d]
    return out


def crc16(data):
    crc = 0xFFFF
    for byte in data:
        crc ^= byte << 8
        for _ in range(8):
            crc = ((crc << 1) ^ 0x1021) & 0xFFFF if crc & 0x8000 else (crc << 1) & 0xFFFF
    return crc


def s(x):
    b = x.encode("utf-8")
    assert len(b) < 256
    return bytes([len(b)]) + b


def head(f, version=1, flags_extra=0):
    flags = f["role"] | (f["secondaryRole"] << 2) | ((1 if f["faction"] == "horde" else 0) << 4)
    if f["itemLevel"] > 0:
        flags |= 0x20
    flags |= flags_extra
    b = bytes([version, flags, f["class"], f["race"], f["level"]])
    b += f["itemLevel"].to_bytes(2, "big") + f["spec"].to_bytes(2, "big")
    b += f["created"].to_bytes(4, "big") + f["counter"].to_bytes(2, "big") + bytes([f["region"]])
    return b


def record(f, version=1):
    return head(f, version) + s(f["realm"]) + s(f["firstName"]) + s(f["surname"]) + s(f["zone"])


def seal(b):
    return b + crc16(b).to_bytes(2, "big")


def base(**kw):
    f = dict(role=1, secondaryRole=0, faction="alliance", itemLevel=412, spec=73, created=29000000, counter=1,
             region=3, realm="3", firstName="Thalion", surname="Brightwood", zone="Westfall", level=24, race=1)
    f["class"] = 1
    f.update(kw)
    return f


def text_of(raw):
    return PREFIX + b45enc(raw)


out = {"_comment": "Generated once with an independent Python codec and segno (QR). Do not edit by hand.", "prefix": PREFIX}

out["base45"] = [
    {"name": n, "hex": d.hex(), "text": b45enc(d)}
    for n, d in [("empty", b""), ("one byte", b"A"), ("two bytes AB (RFC 9285)", b"AB"), ("three bytes", b"abc"),
                 ("Hello!! (RFC 9285, odd 7)", b"Hello!!"), ("base-45 (RFC 9285, odd 7)", b"base-45"),
                 ("ietf! (RFC 9285, odd 5)", b"ietf!"), ("four bytes", bytes([0, 1, 2, 255])),
                 ("max pair", bytes([255, 255])), ("zero pair", bytes([0, 0])), ("single 255", bytes([255])),
                 ("six bytes", bytes(range(10, 16)))]
]
out["base45_errors"] = [
    {"name": "lowercase letter", "text": "ab"},
    {"name": "length mod 3 is 1", "text": "BB8A"},
    {"name": "triple above 65535", "text": "GGW"},
    {"name": "pair above 255", "text": "::"},
    {"name": "outside alphabet", "text": "A#"},
]
out["crc16"] = [{"name": "check string", "hex": b"123456789".hex(), "crc": crc16(b"123456789")},
                {"name": "empty", "hex": "", "crc": crc16(b"")},
                {"name": "single zero", "hex": "00", "crc": crc16(b"\0")}]

recs = []


def add(name, f):
    b = seal(record(f))
    recs.append({"name": name, "fields": f, "hex": b.hex(), "text": text_of(b)})


add("minimal record", base(role=0, itemLevel=0, race=0, level=1, spec=0, created=0, counter=0, region=0,
                           realm="1", firstName="A", surname="B", zone="", **{"class": 0}))
add("typical record", base())
add("maximum string lengths", base(realm="r" * 24, firstName="F" * 24, surname="S" * 24, zone="Z" * 32,
                                   itemLevel=65535, spec=65535, created=4294967295, counter=65535, level=255,
                                   region=255, race=255, **{"class": 255}))
add("accented names", base(firstName="Éloïse", surname="Müller-Ñandú", zone="Vallée des Épreuves", realm="eu-ruleset"))
add("accented names at 24 bytes", base(firstName="é" * 12, surname="ü" * 12, zone="ö" * 16))
add("item level unknown", base(itemLevel=0))
add("item level 1500", base(itemLevel=1500, spec=0))
for r, rn in enumerate(["unknown", "tank", "healer", "damage"]):
    add("role " + rn, base(role=r, counter=10 + r))
for fac in ["alliance", "horde"]:
    add("faction " + fac, base(faction=fac, counter=20))
add("counter 65535", base(counter=65535))
add("counter 0", base(counter=0))
add("zone empty", base(zone=""))
add("numeric realm", base(realm="8"))
sp = None
for c in range(1, 5000):
    f = base(counter=c)
    if " " in text_of(seal(record(f)))[5:]:
        sp = f
        break
add("internal space in text", sp)
out["records"] = recs
space_rec = recs[-1]

errs = []


def err(name, text, error):
    errs.append({"name": name, "text": text, "error": error})


body = record(base())
good = seal(body)
gt = text_of(good)
err("wrong check", text_of(good[:-1] + bytes([good[-1] ^ 1])), "BadCheck")
err("one changed text character (check catches it)", gt[:20] + ("A" if gt[20] != "A" else "B") + gt[21:], "BadCheck")
err("text cut short (under the minimum record)", gt[:35], "Truncated")
err("text cut near the end (check catches it)", gt[:len(gt) - 9], "BadCheck")
err("nothing after the prefix", PREFIX, "Truncated")
err("missing prefix", gt[5:], "BadText")
err("wrong prefix", "WFT2:" + gt[5:], "BadText")
err("lowercase text", gt.lower(), "BadText")
err("length mod 3 is 1", gt + "A", "BadText")
err("character outside the alphabet", gt[:30] + "#" + gt[31:], "BadText")
sp_t = space_rec["text"]
i = sp_t.index(" ", 5)
err("internal space replaced by a dash (check catches it)", sp_t[:i] + "-" + sp_t[i + 1:], "BadCheck")
err("unknown version 2 with valid check", text_of(seal(record(base(), version=2))), "UnknownVersion")
err("unknown version 0 with valid check", text_of(seal(record(base(), version=0))), "UnknownVersion")
err("unknown version, nothing valid behind it", text_of(bytes([2, 1, 2, 3, 4, 5])), "UnknownVersion")
err("trailing byte before the check", text_of(seal(body + b"\x00")), "TrailingBytes")
err("trailing bytes before the check", text_of(seal(body + b"\x01\x02\x03")), "TrailingBytes")
over = bytearray(body)
over[16] = 200
err("realm longer than 24 bytes", text_of(seal(bytes(over))), "InvalidField(realm)")
trunc_body = body[:16] + bytes([1]) + b"3" + bytes([20]) + b"Th" + b"\x00" * 10
err("valid check, first name runs past the end", text_of(seal(trunc_body)), "Truncated")


def mod(idx, fn):
    bb = bytearray(body)
    bb[idx] = fn(bb[idx])
    return text_of(seal(bytes(bb)))


err("level 0", mod(4, lambda v: 0), "InvalidField(level)")
err("reserved flag bit 6 set", mod(1, lambda v: v | 0x40), "InvalidField(flags)")
err("reserved flag bit 7 set", mod(1, lambda v: v | 0x80), "InvalidField(flags)")
f = base()
h = body[:16]
err("empty first name", text_of(seal(h + s(f["realm"]) + bytes([0]) + s(f["surname"]) + s(f["zone"]))), "InvalidField(firstName)")
err("empty surname", text_of(seal(h + s(f["realm"]) + s(f["firstName"]) + bytes([0]) + s(f["zone"]))), "InvalidField(surname)")
err("empty realm", text_of(seal(h + bytes([0]) + s(f["firstName"]) + s(f["surname"]) + s(f["zone"]))), "InvalidField(realm)")
err("zone longer than 32 bytes", text_of(seal(h + s(f["realm"]) + s(f["firstName"]) + s(f["surname"]) + bytes([33]) + b"z" * 33)), "InvalidField(zone)")
err("first name not valid UTF-8", text_of(seal(h + s(f["realm"]) + bytes([3]) + b"\xff\xfe\xfd" + s(f["surname"]) + s(f["zone"]))), "InvalidField(firstName)")
out["decode_errors"] = errs

out["paste"] = [
    {"name": "leading and trailing spaces", "pasted": "  " + gt + "  ", "text": gt},
    {"name": "line breaks and tab", "pasted": "\r\n\t" + gt + "\n", "text": gt},
    {"name": "internal space is kept", "pasted": " " + sp_t + "\n", "text": sp_t},
]

enc_err = []
for name, ff, e in [
    ("neutral faction", base(faction="neutral"), "InvalidField(faction)"),
    ("level 0", base(level=0), "InvalidField(level)"),
    ("level 256", base(level=256), "InvalidField(level)"),
    ("empty surname", base(surname=""), "InvalidField(surname)"),
    ("empty first name", base(firstName=""), "InvalidField(firstName)"),
    ("realm 25 bytes", base(realm="r" * 25), "InvalidField(realm)"),
    ("zone 33 bytes", base(zone="z" * 33), "InvalidField(zone)"),
    ("role 4", base(role=4), "InvalidField(role)"),
    ("item level 65536", base(itemLevel=65536), "InvalidField(itemLevel)"),
    ("counter 65536", base(counter=65536), "InvalidField(counter)"),
]:
    enc_err.append({"name": name, "fields": ff, "error": e})
out["encode_errors"] = enc_err
out["counter_wrap"] = {"from": 65535, "to": 0}
out["limits"] = {"min_record_bytes": 25, "max_record_bytes": 126}


def qr_case(name, text):
    q = segno.make(text, error="m", mode="alphanumeric", micro=False, boost_error=False)
    rows = ["".join(str(v) for v in row) for row in q.matrix]
    other = qrcode.QRCode(version=q.version, error_correction=qrcode.constants.ERROR_CORRECT_M, mask_pattern=q.mask, border=0)
    other.add_data(qrcode.util.QRData(text, mode=qrcode.util.MODE_ALPHA_NUM))
    other.make(fit=False)
    other_rows = ["".join("1" if v else "0" for v in row) for row in other.modules]
    assert other_rows == rows, "second encoder disagrees for " + name
    return {"name": name, "text": text, "version": q.version, "size": len(rows), "mask": q.mask, "rows": rows}


out["qr"] = [
    qr_case("v1 short text", "HELLO WORLD"),
    qr_case("v1 digits and symbols", "0123456789 $%*+-./:"),
    qr_case("v2", "WFT1:" + "ABCDEFGHIJ" * 2),
    qr_case("v3 mid", "A" * 50),
    qr_case("minimal record text", recs[0]["text"]),
    qr_case("typical record text", recs[1]["text"]),
    qr_case("accented record text", recs[3]["text"]),
    qr_case("v5 capacity (122 chars)", "WFT1:" + "Q7" * 58 + "X"),
    qr_case("v6", "7" * 150),
    qr_case("v7 first version with version information", "8" * 170),
    qr_case("maximum string lengths record text", recs[2]["text"]),
    qr_case("v9 capacity (262 chars)", "Y" * 262),
    qr_case("v10 capacity (311 chars)", "Z" * 311),
]
cap = {}
for v in range(1, 11):
    best = 0
    for n in range(1, 400):
        try:
            segno.make("A" * n, error="m", mode="alphanumeric", micro=False, boost_error=False, version=v)
            best = n
        except Exception:
            break
    cap[str(v)] = best
out["qr_capacity_alnum_m"] = cap
out["qr_errors"] = [{"name": "312 characters does not fit version 10", "text": "A" * 312, "error": "TooLong"},
                    {"name": "lowercase is not in the alphanumeric alphabet", "text": "abc", "error": "BadText"},
                    {"name": "empty text", "text": "", "error": "BadText"}]
json.dump(out, open(sys.argv[1], "w"), indent=1, ensure_ascii=False)
print(cap)
print([(q["name"], q["version"], q["mask"]) for q in out["qr"]])
print([(r["name"], len(r["text"]), len(r["hex"]) // 2) for r in recs])
