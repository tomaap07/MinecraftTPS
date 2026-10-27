"""Render original TP branding / UV artwork, never read a Minecraft asset.

First rasterize our SVG:
npm run tauri -- icon static/logos/tp-launcher.svg --fit contain -p 512 -o .codex-tmp/brand-raster
Then run this script with a Python installation containing Pillow.
"""
from pathlib import Path
import sys
import runpy
from PIL import Image, ImageDraw, ImageFont, ImageOps

ROOT = Path(__file__).resolve().parent.parent


def explorer(spectral=False):


    image = Image.new('RGBA', (64, 64))
    d = ImageDraw.Draw(image)
    ink, panel, trim = '#16161E', '#454252', '#A855F7'
    for rect in [(0,0,31,15),(16,16,39,31),(0,16,15,31),(40,16,55,31),(16,48,31,63),(32,48,47,63)]:
        d.rectangle(rect, fill=ink)

    d.rectangle((8,8,15,15),fill=panel)
    d.rectangle((9,10,14,12),fill='#F4F0FF' if spectral else '#7E51C2')
    d.rectangle((10,11,13,11),fill='#FFFFFF' if spectral else '#E9C9FF')
    d.rectangle((1,10,2,12),fill=trim)
    d.rectangle((17,10,18,12),fill=trim)

    for x in (20,32):
        d.rectangle((x,20,x+7,31),fill=panel)
        d.rectangle((x+1,21,x+1,29),fill=trim)
        d.rectangle((x+4,22,x+5,23),fill='#C4B5E8')

    for x,y in ((44,20),(36,52),(4,20),(20,52)):
        d.rectangle((x,y,x+3,y+11),fill=panel)
        d.rectangle((x,y+7,x+3,y+8),fill=trim)
        d.rectangle((x,y+10,x+3,y+11),fill=ink)
    return image


def font(size, bold=False):
    path=Path('C:/Windows/Fonts')/('segoeuib.ttf' if bold else 'segoeui.ttf')
    return ImageFont.truetype(str(path),size) if path.exists() else ImageFont.load_default(size=size)


def surface(width,height):
    im=Image.new('RGB',(width,height),'#0B0B0F');d=ImageDraw.Draw(im)
    for x in range(12,width,28):
        for y in range(14,height,28):
            d.rectangle((x,y,x+19,y+19),outline='#27273A')
    d.rectangle((0,0,3,height),fill='#A855F7')
    return im


def main():
    skins=ROOT/'static/skins';skins.mkdir(exist_ok=True)
    if '--brand-only' not in sys.argv:
        explorer().save(skins/'tp-explorer.png')
        explorer().crop((8,8,16,16)).save(skins/'tp-explorer-face.png')
        explorer(True).save(skins/'tp-spectral.png')

    rasters=list((ROOT/'.codex-tmp/brand-raster').glob('*.png'))
    if not rasters: raise SystemExit('Rasterize static/logos/tp-launcher.svg with the command above first.')
    logo=Image.open(max(rasters,key=lambda path: Image.open(path).width*Image.open(path).height)).convert('RGBA')
    icons=ROOT/'src-tauri/icons'
    for size,name in [(32,'32x32.png'),(128,'128x128.png'),(256,'128x128@2x.png')]:
        logo.resize((size,size),Image.Resampling.LANCZOS).save(icons/name)
    logo.save(icons/'icon.ico',sizes=[(16,16),(24,24),(32,32),(48,48),(64,64),(128,128),(256,256)])
    logo.resize((64,64),Image.Resampling.LANCZOS).save(ROOT/'static/favicon.png')
    installer=runpy.run_path(str(Path(__file__).with_name('prepare-installer-assets.py')))
    installer['main']()
    print('TP branding, favicon, Windows icons and installer graphics generated.' if '--brand-only' in sys.argv else 'Original TP skins, favicon, Windows icons and installer graphics generated.')


if __name__=='__main__': main()
