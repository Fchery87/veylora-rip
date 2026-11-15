// Synthetic fixtures only. No printer, scanner, artwork, or external profile data.
// Run with: node fixtures/images/generate.cjs
const fs = require('node:fs');
const path = require('node:path');
const zlib = require('node:zlib');
function crc(bytes) {
  let c = 0xffffffff;
  for (const b of bytes) {
    c ^= b;
    for (let i = 0; i < 8; i++) c = (c >>> 1) ^ ((c & 1) ? 0xedb88320 : 0);
  }
  return (c ^ 0xffffffff) >>> 0;
}
function chunk(type, data) {
  const b = Buffer.alloc(data.length + 12);
  b.writeUInt32BE(data.length); b.write(type, 4); data.copy(b, 8);
  b.writeUInt32BE(crc(b.subarray(4, -4)), b.length - 4); return b;
}
function image(name, pixels, tags = []) {
  const h = Buffer.alloc(13); h.writeUInt32BE(pixels.length / 4); h.writeUInt32BE(1, 4); h[8] = 8; h[9] = 6;
  const bytes = Buffer.concat([Buffer.from('89504e470d0a1a0a', 'hex'), chunk('IHDR', h), ...tags.map(([t,d]) => chunk(t, d)), chunk('IDAT', zlib.deflateSync(Buffer.from([0, ...pixels]))), chunk('IEND', Buffer.alloc(0))]);
  fs.writeFileSync(path.join(__dirname, name + '.png'), bytes);
}
const pixels = [255, 0, 127, 0, 255, 255, 255, 255];
image('hidden-rgb', pixels);
image('alpha-ramp', [70, 80, 90, 0, 70, 80, 90, 64, 70, 80, 90, 128, 70, 80, 90, 255]);
image('srgb', pixels, [['sRGB', Buffer.from([0])]]);
image('conflicting-color-tags', pixels, [['sRGB', Buffer.from([0])], ['gAMA', Buffer.from('000186a0', 'hex')]]);
image('unsupported-apng', pixels, [['acTL', Buffer.from('0000000100000000', 'hex')]]);
image('unsupported-hdr', pixels, [['cICP', Buffer.from([9,16,0,1])]]);
image('non-square-ppi', pixels, [['pHYs', Buffer.from('00000ec400001d8801', 'hex')]]);
