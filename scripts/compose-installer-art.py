from pathlib import Path
import json
import shutil
import subprocess
import tempfile
from PIL import Image,ImageDraw,ImageFont

ROOT=Path(__file__).resolve().parent.parent
OUT=ROOT/'src-tauri/installer'

def font(size,bold=False):
    file=Path('C:/Windows/Fonts')/('segoeuib.ttf' if bold else 'segoeui.ttf')
    return ImageFont.truetype(str(file),size)

def main():
    version=json.loads((ROOT/'src-tauri/tauri.conf.json').read_text(encoding='utf-8'))['version']
    character=Image.open(OUT/'fariiuh-welcome.png').convert('RGBA')
    character=character.crop(character.getchannel('A').getbbox())
    node=shutil.which('node')
    if not node:raise RuntimeError('Node.js is required to rasterize the project logo.')
    with tempfile.TemporaryDirectory(prefix='tp-installer-logo-') as temp:
        subprocess.run([node,str(ROOT/'node_modules/@tauri-apps/cli/tauri.js'),'icon',str(ROOT/'static/logos/tp-launcher.svg'),'-p','512','-o',temp],cwd=ROOT,check=True,capture_output=True)
        logo=Image.open(Path(temp)/'512x512.png').convert('RGBA')
        for name,width,height in [('sidebar',656,1256),('header',600,228),('skin-install',920,1840)]:
            image=Image.new('RGBA',(width,height),'#100c19')
            draw=ImageDraw.Draw(image)
            draw.rectangle((0,0,5,height),fill='#a855f7')
            if name=='sidebar':
                image.alpha_composite(logo.resize((150,150),Image.Resampling.LANCZOS),(48,38))
                draw.text((48,218),'TP LAUNCHER',font=font(47,True),fill='#f2e5ff')
                draw.text((50,286),'FARIIUH',font=font(24,True),fill='#c084fc')
                picture=character.copy();picture.thumbnail((width-90,800),Image.Resampling.LANCZOS)
                image.alpha_composite(picture,((width-picture.width)//2,355))
                draw.line((48,height-80,width-48,height-80),fill='#583174',width=2)
                draw.text((48,height-59),f'TP LABS  /  v{version}',font=font(24),fill='#bda6d5')
            elif name=='header':
                image.alpha_composite(logo.resize((166,166),Image.Resampling.LANCZOS),(20,20))
                draw.text((206,62),'TP LAUNCHER',font=font(40,True),fill='#f2e5ff')
                draw.text((208,122),f'v{version}',font=font(27),fill='#bda6d5')
            else:
                draw.text((56,65),'TP LAUNCHER',font=font(65,True),fill='#f2e5ff')
                draw.text((58,161),'FARIIUH',font=font(36,True),fill='#c084fc')
                picture=character.copy();picture.thumbnail((width-90,1380),Image.Resampling.LANCZOS)
                image.alpha_composite(picture,((width-picture.width)//2,285))
                draw.line((56,height-130,width-56,height-130),fill='#583174',width=2)
                draw.text((56,height-95),f'TP LABS  /  v{version}',font=font(34),fill='#bda6d5')
            image.convert('RGB').save(OUT/(name+'.bmp'))
            preview=image.copy();preview.thumbnail((460,800),Image.Resampling.LANCZOS);preview.save(OUT/(name+'-preview.png'))
    print('Installer artwork generated with the author-supplied Fariiuh skin render.')

if __name__=='__main__':main()
