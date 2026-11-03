#!/usr/bin/env python3
"""Package the Arcadian Dream content and trusted native Rust source bundle."""
from pathlib import Path
import argparse,json,subprocess,zipfile,shutil,os
ROOT=Path(__file__).resolve().parents[1]
def main():
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('--packager',type=Path,default=(ROOT/'.local/tools/cargo-horizon-mod' if (ROOT/'.local/tools/cargo-horizon-mod').is_file() else Path.home()/'.local/share/horizon-launcher/fork-native/target/fast/cargo-horizon-mod'));p.add_argument('--install',action='store_true');args=p.parse_args()
 subprocess.run(['python3',ROOT/'tools/modernize.py'],check=True)
 subprocess.run(['python3',ROOT/'tools/generate_hooks.py'],check=True)
 subprocess.run(['python3',ROOT/'tools/generate_native.py'],check=True)
 subprocess.run([args.packager,'build'],cwd=ROOT,check=True)
 subprocess.run([args.packager,'check',ROOT/'target/horizon-mod/arcadiandream.hmod','--json'],check=True)
 manifest={'schema':1,'hooks':153,'id':'arcadiandream','version':'0.1.0','game':'26.3','horizon':'d813f63a99978186b127ea14efe5d8603ac2ed4f','mixins':'3f99a5c60dfa311cc365524c44ae2fd799bfa1de'}
 output=ROOT/'target/horizon-mod/arcadiandream.hnmod'
 with zipfile.ZipFile(output,'w',compression=zipfile.ZIP_DEFLATED,compresslevel=9)as archive:
  def add(name,data):
   info=zipfile.ZipInfo(name,(1980,1,1,0,0,0));info.compress_type=zipfile.ZIP_DEFLATED;info.external_attr=0o100644<<16;archive.writestr(info,data)
  add('horizon-native.json',json.dumps(manifest,indent=2).encode())
  for name in ['LICENSE','LICENSE.ASSETS','THIRD_PARTY_NOTICES.md']:add('mixins/arcadiandream/'+('NOTICE.md' if name=='THIRD_PARTY_NOTICES.md' else name),(ROOT/name).read_bytes())
  for file in sorted((ROOT/'native/mixins').rglob('*')):
   if file.is_file():add(file.relative_to(ROOT/'native').as_posix(),file.read_bytes())
 if args.install:
  home=Path(os.environ.get('HORIZON_HOME',Path.home()/'.local/share/horizon'))
  for suffix,folder in [('hmod','horizon-mods'),('hnmod','horizon-native-mods')]:
   dest=home/folder/f'arcadiandream.{suffix}';dest.parent.mkdir(parents=True,exist_ok=True);temp=dest.with_suffix(dest.suffix+'.tmp');shutil.copy2(ROOT/f'target/horizon-mod/arcadiandream.{suffix}',temp);temp.replace(dest)
  selections=home/'horizon-mods.txt';text=selections.read_text()if selections.exists()else''
  if not any(line.strip().startswith('mod.arcadiandream:')for line in text.splitlines()):
   temp=selections.with_suffix('.txt.tmp');temp.write_text(text+(''if not text or text.endswith('\n')else'\n')+'mod.arcadiandream:true\n');temp.replace(selections)
 print('Built',output)
if __name__=='__main__':main()
