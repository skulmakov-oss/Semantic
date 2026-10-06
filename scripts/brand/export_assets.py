"""Offline exports for the deliberately limited flat Semantic SVG vocabulary.

Python 3.11+, Pillow. Optional wordmark regeneration additionally uses fontTools.
No browser, network, font installation, or material-artwork synthesis.
"""
import argparse
import hashlib
import json
import re
import xml.etree.ElementTree as ET
from pathlib import Path

from PIL import Image, ImageDraw

ROOT = Path(__file__).resolve().parents[2]
ASSETS = ROOT / 'assets/brand/semantic/v1.2'
SIZES = (16, 24, 32, 48, 64, 128, 256, 512, 1024)
REFERENCE_SHA = '3f2b97ca2f91a90d5beb4f86489454e80d2750a7b2680683b2b079acc6b6c73d'
FONT_SHA = '0fcbdb5cbeea00ae532352c7c94a7d288ebc911ba85f4d595012032dcab64ba8'
IDENTITY = (1, 0, 0, 1, 0, 0)


def compose(a, b):
    aa, ab, ac, ad, ae, af = a
    ba, bb, bc, bd, be, bf = b
    return (aa*ba+ac*bb, ab*ba+ad*bb, aa*bc+ac*bd,
            ab*bc+ad*bd, aa*be+ac*bf+ae, ab*be+ad*bf+af)


def render(source, size):
    """Render only polygons, rectangles, and straight stroked paths; fail otherwise."""
    tree = ET.parse(source).getroot()
    if tree.get('viewBox') != '0 0 128 128':
        raise ValueError('Flat icon viewBox must be 0 0 128 128')
    scale = size * 4 / 128
    canvas = Image.new('RGBA', (size*4, size*4))
    draw = ImageDraw.Draw(canvas)

    def visit(node, matrix=IDENTITY):
        transform = node.get('transform')
        if transform:
            match = re.fullmatch(r'matrix\(([^)]+)\)', transform)
            if not match:
                raise ValueError('Unsupported flat transform')
            matrix = compose(matrix, tuple(map(float, match[1].split())))
        tag = node.tag.rsplit('}', 1)[-1]
        def point(x, y):
            a, b, c, d, e, f = matrix
            return ((a*x+c*y+e)*scale, (b*x+d*y+f)*scale)
        fill = node.get('fill')
        if fill == 'none':
            fill = None
        stroke = node.get('stroke')
        width = max(1, round(float(node.get('stroke-width', 1))*scale*
                            (abs(matrix[0])+abs(matrix[3]))/2))
        if tag == 'polygon':
            points = [point(*map(float, p.split(','))) for p in node.get('points').split()]
            draw.polygon(points, fill=fill)
            if stroke:
                draw.line(points+[points[0]], fill=stroke, width=width, joint='curve')
        elif tag == 'rect':
            x, y, w, h = (float(node.get(k)) for k in ('x', 'y', 'width', 'height'))
            points = [point(x,y), point(x+w,y), point(x+w,y+h), point(x,y+h)]
            draw.polygon(points, fill=fill)
            if stroke:
                draw.line(points+[points[0]], fill=stroke, width=width, joint='curve')
        elif tag == 'path':
            parts = re.findall(r'[ML]|-?\d+(?:\.\d+)?', node.get('d'))
            if ''.join(parts) != re.sub(r'\s+', '', node.get('d')):
                raise ValueError('Unsupported flat path')
            chain = []
            i = 0
            while i < len(parts):
                command = parts[i]
                if command == 'M' and chain:
                    draw.line(chain, fill=stroke, width=width, joint='curve')
                    chain = []
                chain.append(point(float(parts[i+1]), float(parts[i+2])))
                i += 3
            if chain:
                draw.line(chain, fill=stroke, width=width, joint='curve')
        elif tag not in ('svg', 'g', 'title', 'desc'):
            raise ValueError(f'Unsupported flat element: {tag}')
        for child in node:
            visit(child, matrix)
    visit(tree)
    return canvas.resize((size, size), Image.Resampling.LANCZOS)


def wordmarks(font_path):
    from fontTools.ttLib import TTFont
    from fontTools.pens.svgPathPen import SVGPathPen
    if hashlib.sha256(font_path.read_bytes()).hexdigest() != FONT_SHA:
        raise ValueError('Source font hash differs from approved outlined substitute')
    font = TTFont(font_path)
    glyphs, cmap = font.getGlyphSet(), font.getBestCmap()
    paths, x = [], 0
    for ch in 'semantic':
        name = cmap[ord(ch)]
        pen = SVGPathPen(glyphs)
        glyphs[name].draw(pen)
        paths.append(f'<path transform="translate({x} 0)" d="{pen.getCommands()}"/>')
        x += font['hmtx'][name][0]
    for variant, ink in [('black', '#171819'), ('white', '#FFFFFF')]:
        svg = (f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {x} 1300" '
               'role="img" aria-labelledby="title"><title id="title">semantic</title>'
               f'<g fill="{ink}" transform="translate(0 1200) scale(1 -1)">'+
               ''.join(paths)+'</g></svg>\n')
        (ASSETS/f'wordmark/semantic_wordmark_{variant}.svg').write_text(svg, encoding='utf-8', newline='\n')


def exports():
    for size in SIZES:
        source = ASSETS / ('icons/semantic_app_icon_micro.svg' if size <= 48
                           else 'icons/semantic_app_icon.svg')
        yield source, ASSETS/f'icons/semantic_app_icon_{size}.png', size
    for name, size in [('utility/semantic_utility_mark',512),
                       ('utility/semantic_utility_mark_white',512),
                       ('icons/semantic_file_icon_sm',256)]:
        yield ASSETS/f'{name}.svg', ASSETS/f'{name}.png', size


def validate():
    palette = json.loads((ASSETS/'palette.json').read_text())
    assert palette == dict(zip('NFTS', ['#55575A','#D84A35','#0E9F55','#1769E8']))
    ref = ASSETS/'previews/semantic_visual_identity_v1_2.png'
    assert hashlib.sha256(ref.read_bytes()).hexdigest() == REFERENCE_SHA
    assert Image.open(ref).size == (1448,1086)
    for source in ASSETS.rglob('*.svg'):
        tree = ET.parse(source).getroot()
        assert tree.get('viewBox'), source
        ids = {n.get('id') for n in tree.iter() if n.get('id')}
        for node in tree.iter():
            tag = node.tag.rsplit('}',1)[-1]
            assert tag not in ('image','script','foreignObject','style','text','font'), source
            for key, value in node.attrib.items():
                assert not key.lower().startswith('on'), source
                assert 'http:' not in value and 'https:' not in value and 'data:' not in value, source
                if key.rsplit('}',1)[-1] == 'href':
                    assert value.startswith('#') and value[1:] in ids, source
                for ref_id in re.findall(r'url\(#([^)]+)\)',value):
                    assert ref_id in ids, source
            for ref_id in node.get('aria-labelledby','').split():
                assert ref_id in ids, source
    for source, output, size in exports():
        actual = Image.open(output).convert('RGBA')
        expected = render(source,size)
        assert actual.size == (size,size) and actual.tobytes() == expected.tobytes(), output
    # Check front-plane mapping independently of export filenames.
    for name in ('semantic_app_icon.svg','semantic_app_icon_micro.svg'):
        tree = ET.parse(ASSETS/f'icons/{name}').getroot()
        tiles = [n for n in tree.iter() if n.tag.endswith('rect')]
        assert [(n.get('x'),n.get('y'),n.get('fill')) for n in tiles] == [
            ('4','4',palette['N']),('54','4',palette['F']),
            ('4','54',palette['T']),('54','54',palette['S'])]
    # Independent raster samples: centers of all four tiles at each micro size.
    for size in (16,24,32):
        icon=Image.open(ASSETS/f'icons/semantic_app_icon_{size}.png').convert('RGB')
        for (x,y),state in zip([(74,66),(100,58),(74,96),(100,88)],'NFTS'):
            pixel=icon.getpixel((round(x*size/128),round(y*size/128)))
            target=tuple(bytes.fromhex(palette[state][1:]))
            assert max(abs(a-b) for a,b in zip(pixel,target)) < 65,(size,state,pixel)
    print('PASS: SVG safety/references, palette/order, original reference hash, '
          '12 raster exports, dimensions, rendered pixels, micro quadrant samples')


def contact_sheet():
    sheet=Image.new('RGB',(820,1180),'#E5E7EB')
    draw=ImageDraw.Draw(sheet)
    for row,bg in enumerate(('#FFFFFF','#171819','#FFFFFF','#171819')):
        draw.rectangle((0,row*295,820,(row+1)*295),fill=bg)
        for col,size in enumerate((16,24,32,48,64)):
            icon=(Image.open(ASSETS/f'icons/semantic_app_icon_{size}.png') if row < 2
                  else render(ASSETS/'icons/semantic_file_icon_sm.svg',size))
            x=col*164+25;y=row*295+24
            sheet.paste(icon,(x,y),icon)
            zoom=icon.resize((128,128),Image.Resampling.NEAREST)
            sheet.paste(zoom,(x,row*295+110),zoom)
            draw.text((x,row*295+264),f'{size}px / '+('app' if row < 2 else 'file'),fill='#808080')
    sheet.save(ASSETS/'previews/semantic_icon_scale_checks.png',compress_level=9)


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--check',action='store_true',help='Read-only validation')
    parser.add_argument('--wordmark-font',type=Path)
    args=parser.parse_args()
    if args.check and args.wordmark_font:
        parser.error('--check cannot regenerate wordmarks')
    if not args.check:
        if args.wordmark_font:
            wordmarks(args.wordmark_font)
        for source,output,size in exports():
            render(source,size).save(output,compress_level=9)
        contact_sheet()
    validate()


if __name__ == '__main__':
    main()
