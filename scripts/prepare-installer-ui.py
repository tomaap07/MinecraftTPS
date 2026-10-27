import base64
import ctypes as c
import json
from pathlib import Path
import shutil
import struct
import sys

ROOT=Path(__file__).resolve().parent.parent
OUT=ROOT/'src-tauri/installer/build'

def skip_text(data,index):
    first=struct.unpack_from('<H',data,index)[0]
    if first==0:return index+2
    if first==65535:return index+4
    while struct.unpack_from('<H',data,index)[0]:index+=2
    return index+2

def transform(raw,identifier):
    data=bytearray(raw)
    if data[:4]!=b'\x01\x00\xff\xff':raise ValueError('Expected extended NSIS dialog template')
    count=struct.unpack_from('<H',data,16)[0]
    main=identifier==105
    scale=1.3
    x,y,width,height=struct.unpack_from('<hhhh',data,18)
    struct.pack_into('<hhhh',data,18,round(x*scale),round(y*scale),round(width*scale),round(height*scale)+(18 if main else 0))
    style=struct.unpack_from('<I',data,12)[0]
    if main:struct.pack_into('<I',data,12,(style&~(0x00c00000|0x00040000|0x80))|0x000b0000)
    index=26
    for _ in range(3):index=skip_text(data,index)
    if style&0x40:
        struct.pack_into('<H',data,index,10)
        index=skip_text(data,index+6)
    for _ in range(count):
        index=(index+3)&~3
        x,y,width,height=struct.unpack_from('<hhhh',data,index+12)
        struct.pack_into('<hhhh',data,index+12,round(x*scale),round(y*scale)+(18 if main else 0),round(width*scale),round(height*scale))
        index+=24
        index=skip_text(data,index)
        index=skip_text(data,index)
        extra=struct.unpack_from('<H',data,index)[0]
        index+=2+extra
    return bytes(data)

def main():
    OUT.mkdir(parents=True,exist_ok=True)
    source=ROOT/'src-tauri/installer/dialog-templates.json'
    rows=json.loads(source.read_text(encoding='utf-8'))['dialogs']
    target=OUT/'tp-installer-ui.exe'
    shutil.copyfile(OUT/'TPNative.dll',target)
    kernel=c.WinDLL('kernel32',use_last_error=True)
    kernel.BeginUpdateResourceW.argtypes=[c.c_wchar_p,c.c_int];kernel.BeginUpdateResourceW.restype=c.c_void_p
    kernel.UpdateResourceW.argtypes=[c.c_void_p,c.c_void_p,c.c_void_p,c.c_ushort,c.c_void_p,c.c_uint];kernel.UpdateResourceW.restype=c.c_int
    kernel.EndUpdateResourceW.argtypes=[c.c_void_p,c.c_int];kernel.EndUpdateResourceW.restype=c.c_int
    handle=kernel.BeginUpdateResourceW(str(target),False)
    if not handle:raise c.WinError(c.get_last_error())
    try:
        for identifier,encoded in rows.items():
            data=transform(base64.b64decode(encoded),int(identifier));buffer=c.create_string_buffer(data)
            if not kernel.UpdateResourceW(handle,c.c_void_p(5),c.c_void_p(int(identifier)),0,buffer,len(data)):raise c.WinError(c.get_last_error())
        if not kernel.EndUpdateResourceW(handle,False):raise c.WinError(c.get_last_error())
    except:
        kernel.EndUpdateResourceW(handle,True)
        raise
    print('Prepared enlarged captionless installer dialog resources.')

if __name__=='__main__':main()
