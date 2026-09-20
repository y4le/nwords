"""Exploratory comparison of a qualified development batch prototype and prepared loops."""
import argparse,hashlib,json,os,random,shutil,statistics,subprocess,sys,time
from pathlib import Path
parser=argparse.ArgumentParser();parser.add_argument('--root',type=Path,required=True);args=parser.parse_args()
script=Path(__file__).resolve()
root=args.root.resolve();os.chdir(root);os.sched_setaffinity(0,{19})
sys.path.insert(0,str(root/'benchmarks'))
from run import cpu_ticks
from report import summarize
os.environ.pop('NODE_OPTIONS',None);os.environ.pop('NODE_COMPILE_CACHE',None)
build=json.loads((root/'dist/package-build.json').read_text())
qualification=json.loads((root/'dist/package-qualification.json').read_text())
if qualification['tarballSha256'] != build['tarballSha256']: raise RuntimeError('Qualification mismatch')
consumer=root/'benchmarks/.work/batch-consumer'
if consumer.exists(): shutil.rmtree(consumer)
consumer.mkdir(parents=True);(consumer/'package.json').write_text('{"private":true,"type":"module"}\n')
subprocess.run(['npm','install','--ignore-scripts','--no-audit','--no-fund',build['tarball']],cwd=consumer,check=True,stdout=subprocess.DEVNULL)
package=consumer/'node_modules/@y4le/nwords'
subprocess.run(['cargo','build','--locked','--release','--manifest-path','benchmarks/rust/Cargo.toml'],env=dict(os.environ,RUSTUP_TOOLCHAIN='1.94.0'),check=True)
reference=subprocess.check_output(['benchmarks/rust/target/release/nwords-comparison','vectors','0','100','benchmarks/ids-u32.txt'])
(root/'benchmarks/.work/native-phrases.tsv').write_bytes(reference)

assert hashlib.sha256((package/'wasm/nwords_js_bg.wasm').read_bytes()).hexdigest()==build['wasm']['sha256']
metadata={'sourceCommit':subprocess.check_output(['git','rev-parse','HEAD'],text=True).strip(),'development':True,'artifact':build,'qualification':qualification,'nativeReferenceSha256':hashlib.sha256(reference).hexdigest(),'fixtureSha256':hashlib.sha256((root/'benchmarks/ids-u32.txt').read_bytes()).hexdigest(),'cpuInfo':subprocess.check_output(['lscpu'],text=True),'roundLoad':[],'cpu':19,'rounds':3,'samplesPerProcess':3,'warmupMs':100,'targetSampleMs':100,'startedUtc':time.strftime('%Y-%m-%dT%H:%M:%SZ',time.gmtime())}
metadata['diffSha256']=hashlib.sha256(subprocess.check_output(['git','diff','HEAD'])).hexdigest()
metadata['harnessHashes']={str(p):hashlib.sha256(p.read_bytes()).hexdigest() for p in [script,root/'benchmarks/js/suite.mjs',root/'benchmarks/js/node.mjs',root/'benchmarks/js/browser.mjs']}
records=[]
for round in range(3):
 load={'round':round,'before':os.getloadavg(),'cpuTicksBefore':cpu_ticks(19)}
 cells=[(runtime,kind,size,op) for runtime in ['node','browser'] for kind in ['batch','loop'] for size in [1,16,256] for op in ['encode','decode']]
 random.Random(8931+round).shuffle(cells);print('round',round+1,flush=True)
 for runtime,kind,size,op in cells:
  rows=subprocess.check_output(['node',f'benchmarks/js/{runtime}.mjs',str(package),f'nwords-{kind}/batch-{size}/{op}',str(round),'100'],text=True)
  records.extend(json.loads(row) for row in rows.splitlines())
 load.update(after=os.getloadavg(),cpuTicksAfter=cpu_ticks(19));metadata['roundLoad'].append(load)
if hashlib.sha256(subprocess.check_output(['git','diff','HEAD'])).hexdigest()!=metadata['diffSha256']: raise RuntimeError('Prototype changed')
for name,digest in metadata['harnessHashes'].items():
 if hashlib.sha256(Path(name).read_bytes()).hexdigest()!=digest: raise RuntimeError('Harness changed')
summarize({'metadata':{'rounds':3},'records':records})
metadata['completedUtc']=time.strftime('%Y-%m-%dT%H:%M:%SZ',time.gmtime())
(root/'benchmarks/.work/batch-experiment.json').write_text(json.dumps({'metadata':metadata,'records':records},indent=2)+'\n')
for runtime in ['node','chromium']:
 for size in [1,16,256]:
  for op in ['encode','decode']:
   medians={kind:statistics.median(statistics.median(r['nsPerOp'] for r in records if r['kind']=='sample' and r['runtime']==runtime and r['case']==f'nwords-{kind}/batch-{size}/{op}' and r['round']==round) for round in range(3))/size for kind in ['batch','loop']}
   print(runtime,size,op,medians,flush=True)
