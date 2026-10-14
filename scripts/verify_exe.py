"""Inspect a Windows distribution without executing it (standard library only)."""
import hashlib
import pathlib
import struct
import sys

path = pathlib.Path(sys.argv[1])
data = path.read_bytes()
pe = struct.unpack_from('<I', data, 0x3C)[0]
assert data[pe:pe + 4] == b'PE\0\0', 'not a PE executable'
machine, count = struct.unpack_from('<HH', data, pe + 4)
optional_size = struct.unpack_from('<H', data, pe + 20)[0]
optional = pe + 24
assert machine == 0x8664, 'not x64'
assert struct.unpack_from('<H', data, optional)[0] == 0x20B, 'not PE32+'
assert struct.unpack_from('<H', data, optional + 68)[0] == 2, 'not GUI subsystem'
sections = []
for index in range(count):
    offset = optional + optional_size + 40 * index
    virtual_size, address, raw_size, raw = struct.unpack_from('<IIII', data, offset + 8)
    sections.append((address, max(virtual_size, raw_size), raw))


def file_offset(address):
    for start, size, raw in sections:
        if start <= address < start + size:
            return raw + address - start
    raise ValueError(f'unmapped RVA: {address}')


imports = []
offset = file_offset(struct.unpack_from('<I', data, optional + 120)[0])
while any(data[offset:offset + 20]):
    name = file_offset(struct.unpack_from('<I', data, offset + 12)[0])
    imports.append(data[name:data.index(0, name)].decode('ascii'))
    offset += 20
assert not any(any(token in dll.lower() for token in ('vcruntime', 'msvcp', 'api-ms-win-crt', 'mscoree', 'hostfxr')) for dll in imports), 'unexpected runtime dependency'
resources = file_offset(struct.unpack_from('<I', data, optional + 128)[0])
named, ids = struct.unpack_from('<HH', data, resources + 12)
types = [struct.unpack_from('<I', data, resources + 16 + index * 8)[0] for index in range(named + ids)]
assert {3, 14, 24}.issubset(types), 'missing icon/group icon/manifest'
digest = hashlib.sha256(data).hexdigest()
if len(sys.argv) > 2:
    expected = pathlib.Path(sys.argv[2]).read_text().split()[0]
    assert digest == expected, 'checksum mismatch'
print(f'PASS: x64 GUI, embedded icon/manifest, standalone runtime; {len(data)} bytes')
print(f'{digest}  {path.name}')
print('Imports: ' + ', '.join(imports))
