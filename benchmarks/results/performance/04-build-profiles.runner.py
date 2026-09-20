"""Compare qualified packages from one clean source; reuse the ordinary JS cells."""
import argparse,hashlib,json,os,random,shutil,statistics,subprocess,time
from pathlib import Path
parser=argparse.ArgumentParser()
parser.add_argument('--root',type=Path,required=True)
parser.add_argument('--wasm-opt',required=True)
args=parser.parse_args()
ROOT=args.root.resolve()
WORK=ROOT/'benchmarks/.work/profiles'
OPT=str(Path(args.wasm_opt).resolve())
os.chdir(ROOT)
os.environ.pop('NODE_COMPILE_CACHE',None)
os.environ.pop('NODE_OPTIONS',None)
os.sched_setaffinity(0,{19})
def output(cmd,**kw): return subprocess.check_output(cmd,text=True,**kw).strip()
def digest(p): return hashlib.sha256(p.read_bytes()).hexdigest()
assert not output(['git','status','--porcelain=v1','--untracked-files=all'])
commit=output(['git','rev-parse','HEAD'])
WORK.mkdir(exist_ok=True,parents=True)
variants={}
for name,profile,optimized in [('baseline','baseline',False),('thin','thin',False),('fat','fat',False),('size','size',False),('fat-opt','fat',True)]:
 print('Build and qualify',name,flush=True)
 with (WORK/f'{name}-build.log').open('w') as log:
  subprocess.run(['node','packages/nwords-js/scripts/build.mjs','--profile',profile]+(['--wasm-opt',OPT] if optimized else []),stdout=log,stderr=subprocess.STDOUT,check=True)
 with (WORK/f'{name}-qualification.log').open('w') as log:
  subprocess.run(['node','packages/nwords-js/scripts/test-package.mjs'],stdout=log,stderr=subprocess.STDOUT,check=True)
 metadata=json.loads((ROOT/'dist/package-build.json').read_text())
 assert not metadata['dirty'] and not metadata['development'] and metadata['sourceCommit']==commit
 tarball=WORK/f'{name}.tgz'; shutil.copyfile(metadata['tarball'],tarball)
 consumer=WORK/name
 if consumer.exists(): shutil.rmtree(consumer)
 consumer.mkdir(); (consumer/'package.json').write_text('{"private":true,"type":"module"}\n')
 subprocess.run(['npm','install','--ignore-scripts','--no-audit','--no-fund',str(tarball)],cwd=consumer,stdout=subprocess.DEVNULL,check=True)
 package=consumer/'node_modules/@y4le/nwords'
 assert digest(package/'wasm/nwords_js_bg.wasm')==metadata['wasm']['sha256']
 variants[name]={'package':str(package),'build':metadata,'qualificationLogSha256':digest(WORK/f'{name}-qualification.log')}
paths=[ROOT/f'benchmarks/js/{name}.mjs' for name in ['node','browser','suite','cold']]+[Path(__file__)]
hashes={str(p):digest(p) for p in paths}
data={'metadata':{'sourceCommit':commit,'sourceDirty':False,'cpu':19,'startedUtc':time.strftime('%Y-%m-%dT%H:%M:%SZ',time.gmtime()),'rounds':7,'samplesPerProcess':3,'warmupMs':100,'targetSampleMs':100,'fixtureSha256':digest(ROOT/'benchmarks/ids-u32.txt'),'harnessHashes':hashes,'variants':variants,'order':'Seeded shuffle across profiles, runtimes and operations each round','cpuInfo':output(['lscpu']),'node':output(['node','--version']),'npmLockSha256':digest(ROOT/'benchmarks/js/package-lock.json')},'records':[]}
def save(): (WORK/'results.json').write_text(json.dumps(data,indent=2)+'\n')
for round in range(7):
 cells=[(name,runtime,op) for name in variants for runtime in ['node','browser'] for op in ['encode','decode']]
 random.Random(9931+round).shuffle(cells)
 print('Warm round',round+1,flush=True)
 for name,runtime,op in cells:
  rows=output(['node',f'benchmarks/js/{runtime}.mjs',variants[name]['package'],f'nwords/u32/{op}',str(round),'100'])
  data['records'].extend(dict(profile=name,**json.loads(row)) for row in rows.splitlines())
 save()
for round in range(30):
 names=list(variants); random.Random(6123+round).shuffle(names)
 for name in names:
  row=json.loads(output(['node','benchmarks/js/cold.mjs','nwords',variants[name]['package']]))
  data['records'].append(dict(profile=name,round=round,**row))
assert output(['git','rev-parse','HEAD'])==commit and not output(['git','status','--porcelain=v1','--untracked-files=all'])
assert all(digest(p)==hashes[str(p)] for p in paths)
data['metadata']['completedUtc']=time.strftime('%Y-%m-%dT%H:%M:%SZ',time.gmtime());save()
for name in variants:
 cells={}
 for runtime in ['node','chromium']:
  for op in ['encode','decode']:
   medians=[statistics.median(r['nsPerOp'] for r in data['records'] if r['kind']=='sample' and r['profile']==name and r['runtime']==runtime and r['case']==f'nwords/u32/{op}' and r['round']==round) for round in range(7)]
   assert len(medians)==7
   cells[f'{runtime}/{op}']=statistics.median(medians)
 print(name,variants[name]['build']['wasm']['bytes'],cells,flush=True)
