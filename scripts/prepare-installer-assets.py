from pathlib import Path
import runpy

def main():
    runpy.run_path(str(Path(__file__).with_name('compose-installer-art.py')))['main']()

if __name__=='__main__':main()
