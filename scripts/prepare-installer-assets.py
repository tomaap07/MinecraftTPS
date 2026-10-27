from pathlib import Path
import json
import shutil
import subprocess
import tempfile
from PIL import Image, ImageDraw, ImageFont

ROOT = Path(__file__).resolve().parent.parent
OUT = ROOT / 'src-tauri/installer'
MASCOT = '''<svg xmlns="http://www.w3.org/2000/svg" width="280" height="420" viewBox="0 0 280 420" fill="none">
<defs><linearGradient id="suit" x1="70" y1="170" x2="200" y2="365" gradientUnits="userSpaceOnUse"><stop stop-color="#514765"/><stop offset="1" stop-color="#1F192E"/></linearGradient><linearGradient id="visor" x1="80" y1="75" x2="180" y2="120" gradientUnits="userSpaceOnUse"><stop stop-color="#E2BEFF"/><stop offset=".45" stop-color="#A855F7"/><stop offset="1" stop-color="#65229E"/></linearGradient></defs>
<ellipse cx="145" cy="392" rx="95" ry="17" fill="#A855F7" opacity=".14"/>
<path d="m94 173-20 9-7 100 18 12 22-14 5-96Z" fill="#171220" stroke="#625073" stroke-width="3"/>
<path d="m110 276-11 85 33 9 17-79 10 77 33-6-7-92Z" fill="#211B31" stroke="#50435F" stroke-width="3"/>
<path d="m104 315 30 7-3 9-29-6m58-3 30-5 1 9-30 5" fill="#A855F7"/>
<path d="m99 359-12 22 5 9 40 3 11-12-9-14Z" fill="#353044" stroke="#746381" stroke-width="3"/>
<path d="m159 366-7 17 12 11 44-7 4-10-20-17Z" fill="#353044" stroke="#746381" stroke-width="3"/>
<path d="m87 383 44 4m26 2 49-7" stroke="#A855F7" stroke-width="4"/>
<path d="m96 169 47-15 48 16 9 108-53 21-56-19Z" fill="url(#suit)" stroke="#887299" stroke-width="3"/>
<path d="m106 177 7 92 14 6-7-101Z" fill="#A855F7"/>
<path d="m137 197 25-8 16 10-25 10Z" fill="#D9BAF7"/><path d="m153 209 25-10v19l-25 10Z" fill="#AA4AF0"/><path d="m137 197 16 12v19l-16-12Z" fill="#7233A5"/>
<path d="m129 254 42-13 8 16-43 15Z" fill="#17121F" stroke="#736083" stroke-width="2"/>
<path d="m175 171 24-1 28 42-12 17-32-27Z" fill="#45394F" stroke="#867193" stroke-width="3"/>
<path d="m211 212 21 5-5 49-19 7-10-17Z" fill="#292231" stroke="#6B5877" stroke-width="3"/>
<path d="m207 247 22 4-2 10-22-4" fill="#A855F7"/>
<path d="m207 265-7 15 9 14 16-4 7-14-7-12Z" fill="#322A3C" stroke="#8E789E" stroke-width="3"/>
<path d="m98 174-21-3-29 35 14 19 30-23Z" fill="#45394F" stroke="#867193" stroke-width="3"/>
<path d="m50 210-17-32 19-36 17 10-10 33 7 27Z" fill="#3B3049" stroke="#8E789E" stroke-width="3"/>
<path d="m37 174 21 12 4-10-19-12" fill="#A855F7"/>
<path d="m45 149-15-13-6-21 6-6 11 14-4-31 7-3 8 27 1-31 8 1 2 31 9-22 7 5-8 27-10 25Z" fill="#3E334A" stroke="#9A81AB" stroke-width="3"/>
<path d="m97 139 6 29 42 13 38-14 3-30Z" fill="#20182C" stroke="#645170" stroke-width="3"/>
<path d="m99 144 43 14 42-14" stroke="#BC6DFC" stroke-width="5"/>
<path d="m96 40 53-11 46 24 10 71-24 30-42 10-49-23-15-58Z" fill="#2D233C" stroke="#AA8AC1" stroke-width="3"/>
<path d="m96 40 50 12 49 1-46-24Z" fill="#5F4D71"/>
<path d="m146 52-7 112 42-10 24-30-10-71Z" fill="#1B1525"/>
<path d="m89 75 50-13 48 14 7 40-18 18-38 10-42-18Z" fill="#08070D" stroke="#7F6593" stroke-width="3"/>
<path d="m96 82 44-12 39 12 5 28-13 16-32 9-35-16Z" fill="url(#visor)"/>
<path d="m106 86 35-9 27 9-29 4-30 10Z" fill="#F4E8FF" opacity=".82"/>
<path d="m102 115 35 12 35-10" stroke="#D9B7F6" stroke-width="3" opacity=".5"/>
<path d="m84 88-8 6 5 25 11 5m103-39 10 5 3 29-10 7" fill="#5C456E" stroke="#AD8EC1" stroke-width="3"/>
<path d="m80 99 7 20m115-25 2 24" stroke="#D67BFF" stroke-width="4"/>
</svg>'''


def font(size, bold=False):
    source = Path('C:/Windows/Fonts') / ('segoeuib.ttf' if bold else 'segoeui.ttf')
    return ImageFont.truetype(str(source), size) if source.exists() else ImageFont.load_default(size=size)


def frame(width, height, content):
    return f'''<svg xmlns="http://www.w3.org/2000/svg" width="{width}" height="{height}" viewBox="0 0 {width} {height}"><defs><radialGradient id="glow"><stop stop-color="#62258F" stop-opacity=".4"/><stop offset="1" stop-color="#100C19" stop-opacity="0"/></radialGradient></defs><rect width="100%" height="100%" fill="#100C19"/><ellipse cx="{width*.6}" cy="{height*.57}" rx="{width*.9}" ry="{height*.39}" fill="url(#glow)"/>{content}</svg>'''


def main():
    version = json.loads((ROOT/'src-tauri/tauri.conf.json').read_text(encoding='utf-8'))['version']
    logo = (ROOT/'static/logos/tp-launcher.svg').read_text(encoding='utf-8')
    logo = logo.split('>',1)[1].rsplit('</svg>',1)[0]
    mascot = MASCOT.replace('<svg ', '<svg x="0" y="0" ', 1)
    OUT.mkdir(parents=True, exist_ok=True)
    (OUT/'tp-explorer.svg').write_text(MASCOT, encoding='utf-8')
    cli = ROOT/'node_modules/@tauri-apps/cli/tauri.js'
    node = shutil.which('node')
    if not node or not cli.is_file():
        raise SystemExit('Node.js and the local Tauri CLI are required to render original SVG artwork.')
    sources = {
        'sidebar': (328, 628, frame(328, 628,
            '<path d="M0 0h4v628H0Z" fill="#A855F7"/>'
            '<path d="m24 221 140-80 140 80v225l-140 80-140-80Z" stroke="#683C86" stroke-width="2" fill="none" opacity=".5"/>'
            '<path d="m12 440 152 87 152-87m-304 40 152 87 152-87" stroke="#B47CDF" fill="none" opacity=".18"/>'
            f'<svg x="24" y="24" width="86" height="86" viewBox="0 0 256 256">{logo}</svg>'
            f'<g transform="translate(22 191) scale(1.02)">{mascot}</g>'
            '<path d="M24 584h280" stroke="#614074"/>'
            '<path d="m273 142 4-10 4 10 10 4-10 4-4 10-4-10-10-4Z" fill="#C376FC"/>')),
        'header': (300, 114, frame(300, 114,
            f'<svg x="14" y="13" width="86" height="86" viewBox="0 0 256 256">{logo}</svg>'
            '<path d="M0 112h300" stroke="#653D85"/>')),
        'skin-install': (230, 460, frame(230, 460,
            '<path d="m20 140 95-55 95 55v205l-95 55-95-55Z" stroke="#743B98" fill="none" opacity=".55"/>'
            f'<g transform="translate(-4 75) scale(.86)">{mascot}</g>'
            '<path d="M18 421h194" stroke="#614074"/>')),
    }
    with tempfile.TemporaryDirectory(prefix='tp-installer-art-') as temp:
        temp = Path(temp)
        for name, (width, height, svg) in sources.items():
            side = max(width, height)
            x, y = (side-width)//2, (side-height)//2
            square = f'<svg xmlns="http://www.w3.org/2000/svg" width="{side}" height="{side}" viewBox="0 0 {side} {side}"><svg x="{x}" y="{y}" width="{width}" height="{height}" viewBox="0 0 {width} {height}">{svg.split(">",1)[1].rsplit("</svg>",1)[0]}</svg></svg>'
            source = temp/f'{name}.svg'
            source.write_text(square, encoding='utf-8')
            destination = temp/name
            subprocess.run([node, str(cli), 'icon', str(source), '-p', str(side), '-o', str(destination)], cwd=ROOT, check=True, capture_output=True)
            with Image.open(destination/f'{side}x{side}.png') as rendered:
                image = rendered.crop((x,y,x+width,y+height)).convert('RGB')
            draw = ImageDraw.Draw(image)
            if name == 'sidebar':
                draw.text((24,123), 'TP LAUNCHER', font=font(28,True), fill='#F5EAFE')
                draw.text((25,162), 'TU SIGUIENTE AVENTURA.', font=font(15), fill='#BCA7CE')
                draw.text((24,594), f'TP STUDIOS  /  v{version}', font=font(14,True), fill='#C3A7DB')
                image = image.resize((164,314),Image.Resampling.LANCZOS)
            elif name == 'header':
                draw.text((113,30), 'TP LAUNCHER', font=font(22,True), fill='#F5EAFE')
                draw.text((114,63), f'INSTALACIÓN  ·  v{version}', font=font(13), fill='#BCA7CE')
                image = image.resize((150,57),Image.Resampling.LANCZOS)
            else:
                draw.text((18,20), 'TU PRÓXIMA', font=font(17,True), fill='#F5EAFE')
                draw.text((18,43), 'AVENTURA.', font=font(24,True), fill='#C480FF')
                draw.text((18,432), 'TP STUDIOS  /  EXPLORER', font=font(11,True), fill='#C3A7DB')
            image.save(OUT/f'{name}.bmp')
            image.save(OUT/f'{name}-preview.png')
    print(f'Original TP installer artwork generated for v{version}: header, sidebar and Explorer copy panel.')


if __name__ == '__main__':
    main()
