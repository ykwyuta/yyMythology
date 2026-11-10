"""Natural Earth (public domain) の海岸線・湖・河川を、ユーラシア用の Rust データに変換する。
usage: python3 -I geoprep.py <ne_dir> <out.rs>
"""
import json, sys, math

NE, OUT = sys.argv[1], sys.argv[2]
LON0, LON1, LAT0, LAT1 = -26.0, 192.0, -12.0, 83.0


def clip(ring, edge):
    # Sutherland-Hodgman, edge=(axis, value, keep_greater)
    ax, v, ge = edge
    out = []
    n = len(ring)
    if n == 0:
        return out
    def inside(p):
        return p[ax] >= v if ge else p[ax] <= v
    def inter(a, b):
        t = (v - a[ax]) / (b[ax] - a[ax])
        return (a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t)
    prev = ring[-1]
    for cur in ring:
        if inside(cur):
            if not inside(prev):
                out.append(inter(prev, cur))
            out.append(cur)
        elif inside(prev):
            out.append(inter(prev, cur))
        prev = cur
    return out


def clip_box(ring):
    for e in [(0, LON0, True), (0, LON1, False), (1, LAT0, True), (1, LAT1, False)]:
        ring = clip(ring, e)
    return ring


def dp(pts, tol):
    if len(pts) < 3:
        return pts
    a, b = pts[0], pts[-1]
    dx, dy = b[0] - a[0], b[1] - a[1]
    l = math.hypot(dx, dy)
    best, bi = -1, 0
    for i in range(1, len(pts) - 1):
        p = pts[i]
        if l == 0:
            d = math.hypot(p[0] - a[0], p[1] - a[1])
        else:
            d = abs(dy * p[0] - dx * p[1] + b[0] * a[1] - b[1] * a[0]) / l
        if d > best:
            best, bi = d, i
    if best > tol:
        return dp(pts[: bi + 1], tol)[:-1] + dp(pts[bi:], tol)
    return [a, b]


def area(r):
    s = 0
    for i in range(len(r)):
        x0, y0 = r[i]
        x1, y1 = r[(i + 1) % len(r)]
        s += x0 * y1 - x1 * y0
    return abs(s) / 2


def rings_of(g):
    if g['type'] == 'Polygon':
        return g['coordinates']
    if g['type'] == 'MultiPolygon':
        return [r for p in g['coordinates'] for r in p]
    return []


def simp_ring(r, tol):
    r = [(p[0], p[1]) for p in r]
    if r[0] == r[-1]:
        r = r[:-1]
    r = clip_box(r)
    if len(r) < 3:
        return None
    # 閉じた輪を2つに割って簡略化
    h = len(r) // 2
    s = dp(r[: h + 1], tol)[:-1] + dp(r[h:] + [r[0]], tol)[:-1]
    if len(s) < 3 or area(s) < 0.04:
        return None
    return s


# Use country geometry only to select the playable land. Borders are never drawn.
# Keep the original shared vertices so neighboring polygons join without seams.
countries = json.load(open(f'{NE}/ne_50m_admin_0_countries.geojson', encoding='utf-8'))
land_rings = []
for f in countries['features']:
    props = f['properties']
    if props['CONTINENT'] not in {'Europe', 'Asia'} and props['ADMIN'] != 'Egypt':
        continue
    geom = f['geometry']
    polygons = geom['coordinates'] if geom['type'] == 'MultiPolygon' else [geom['coordinates']]
    for outer, *_ in polygons:
        if props['ADMIN'] in {'Spain', 'Portugal'} and max(p[1] for p in outer) < 35.9:
            continue  # North African territories and Atlantic islands off Africa
        r = [(lon + 360 if lon < -160 else lon, lat) for lon, lat, *_ in outer]
        r = clip_box(r[:-1])
        if len(r) >= 3 and area(r) >= 0.025:
            land_rings.append(r)

LAKES = {'Lake Ladoga', 'Lake Baikal', 'Lake Onega', 'Issyk-Kul', 'Lake Zaysan', 'Khövsgöl Nuur', 'Uvs Nuur', 'Lake Khanka',
         'Qinghai Hu', 'Lake Urmia', 'Hulun Nuur', 'Lake Peipus', 'Tai Hu', 'Poyang Hu', 'Lake Van', 'Lake Sevan',
         'South Aral Sea', 'North Aral Sea', 'Lake Balkhash', 'Har Us Nuur', 'Khyargas Nuur', 'Alaköl', 'Lake Tuz'}
lakes = json.load(open(f'{NE}/ne_50m_lakes.geojson'))
lake_rings = []
seen = set()
for f in lakes['features']:
    n = f['properties'].get('name')
    if n not in LAKES and n != 'Caspian Sea':
        continue
    for r in rings_of(f['geometry'])[:1]:
        s = simp_ring(r, 0.04)
        key = (n, round(s[0][0], 2) if s else 0)
        if s and key not in seen:
            seen.add(key)
            lake_rings.append((n, s))

# 河川：名前 → 大河か
RIV = {
    'Huang': True, 'Chang Jiang': True, 'Yangtze': True, 'Han': False, 'Amur': True, 'Heilong Jiang': True, 'Songhua': False,
    'Liao': False, 'Xiliao': False, 'Selenge (Selenga)': False, 'Irtysh': False, 'Ertis': False, 'Ertix': False,
    'Syr Darya': True, 'Amu  Darya': True, 'Panj': False, 'Volga': True, 'Kama': False, 'Don': True, 'Dnipro': True, 'Dnepre': True,
    'Dniester': False, 'Danube': True, 'Donau': True, 'Tisza': False, 'Tisa': False, 'Vistula': False, 'Oder': False,
    'Tigris': True, 'Dicle': True, 'Euphrates': True, 'Firat': True, 'Al Furat': True, 'Shatt al Arab': True,
    'Nile': True, 'Rosetta Branch': False, 'Damietta Branch': False, 'Indus': True, 'Ganges': True, 'Hong': False, 'Xi': False,
    'Gan': False, 'Yuan': False, 'Ural': False, 'Tarim': False, 'Daugava': False, 'Neva': False, 'Ob': True, 'Lena': True, 'Yenisey': True, 'Mekong': True, 'Irrawaddy': True,
    'Argun’': False, 'Hailar': False, 'Naryn': False, 'Chenab': False, 'Sutlej': False, 'Yamuna': False, 'Jinsha': False,
}
rivers = json.load(open(f'{NE}/ne_50m_rivers_lake_centerlines.geojson'))
riv_out = []
for f in rivers['features']:
    n = f['properties'].get('name')
    if n not in RIV:
        continue
    g = f['geometry']
    cs = g['coordinates'] if g['type'] == 'MultiLineString' else [g['coordinates']]
    for c in cs:
        pts = [(p[0], p[1]) for p in c if LON0 <= p[0] <= LON1 and LAT0 <= p[1] <= LAT1]
        if len(pts) < 2:
            continue
        s = dp(pts, 0.05)
        riv_out.append((n, RIV[n], s))


def fmt(pts):
    return '&[' + ', '.join(f'({x:.2f}, {y:.2f})' for x, y in pts) + ']'

with open(OUT, 'w') as o:
    o.write('//! 地理データ（自動生成：tools/geoprep.py）。\n')
    o.write('//! 出典：Natural Earth（パブリックドメイン）の 1:50m 陸地・湖・河川を簡略化したもの。\n\n')
    o.write('/// 陸地の輪（外周と内側の穴。偶奇規則で塗る）\n')
    o.write('pub const LAND_RINGS: &[&[(f32, f32)]] = &[\n')
    for r in land_rings:
        o.write('    ' + fmt(r) + ',\n')
    o.write('];\n\n/// 湖\n')
    o.write('pub const LAKE_RINGS: &[&[(f32, f32)]] = &[\n')
    for n, r in lake_rings:
        o.write(f'    // {n}\n    ' + fmt(r) + ',\n')
    o.write('];\n\n/// 河川（大河か, 折れ線）\n')
    o.write('pub const NE_RIVERS: &[(bool, &[(f32, f32)])] = &[\n')
    for n, m, s in riv_out:
        o.write(f'    // {n}\n    ({"true" if m else "false"}, ' + fmt(s) + '),\n')
    o.write('];\n')

print('land rings', len(land_rings), sum(len(r) for r in land_rings))
print('lakes', len(lake_rings), sum(len(r) for _, r in lake_rings))
print('rivers', len(riv_out), sum(len(s) for _, _, s in riv_out))
