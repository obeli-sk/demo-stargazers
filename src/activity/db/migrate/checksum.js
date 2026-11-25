// refinery's migration checksum: SipHash-1-3 with zero keys over Rust's `Hash` of name, version and SQL.

const MASK = (1n << 64n) - 1n;
const rotl = (x, bits) => ((x << BigInt(bits)) | (x >> BigInt(64 - bits))) & MASK;

function sipHash13(bytes) {
    let v0 = 0x736f6d6570736575n, v1 = 0x646f72616e646f6dn, v2 = 0x6c7967656e657261n, v3 = 0x7465646279746573n;
    const round = () => {
        v0 = (v0 + v1) & MASK; v1 = rotl(v1, 13); v1 ^= v0; v0 = rotl(v0, 32);
        v2 = (v2 + v3) & MASK; v3 = rotl(v3, 16); v3 ^= v2;
        v0 = (v0 + v3) & MASK; v3 = rotl(v3, 21); v3 ^= v0;
        v2 = (v2 + v1) & MASK; v1 = rotl(v1, 17); v1 ^= v2; v2 = rotl(v2, 32);
    };
    // Little-endian word of `length` bytes starting at `start`.
    const word = (start, length) => {
        let m = 0n;
        for (let j = length - 1; j >= 0; j--) m = (m << 8n) | BigInt(bytes[start + j]);
        return m;
    };
    const full = bytes.length - (bytes.length % 8);
    for (let i = 0; i < full; i += 8) {
        const m = word(i, 8);
        v3 ^= m; round(); v0 ^= m;
    }
    const m = (BigInt(bytes.length & 0xff) << 56n) | word(full, bytes.length - full);
    v3 ^= m; round(); v0 ^= m;
    v2 ^= 0xffn; round(); round(); round();
    return (v0 ^ v1 ^ v2 ^ v3) & MASK;
}

// The checksum as the decimal string stored in the history table.
// Rust hashes a `str` as its bytes and 0xff, an `i32` as 4 little-endian bytes.
export function checksum(name, version, sql) {
    const encoder = new TextEncoder();
    const versionBytes = new Uint8Array(4);
    new DataView(versionBytes.buffer).setInt32(0, version, true);
    const bytes = [...encoder.encode(name), 0xff, ...versionBytes, ...encoder.encode(sql), 0xff];
    return sipHash13(Uint8Array.from(bytes)).toString();
}
