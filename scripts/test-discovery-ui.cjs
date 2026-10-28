// Optional integration check: real generated scripts in jsdom plus real CLI import.
// No browser layout, clipboard permission or file-download behavior is certified.
const { JSDOM, VirtualConsole } = require('jsdom');
const fs = require('node:fs');
const path = require('node:path');
const os = require('node:os');
const cp = require('node:child_process');
const assert = require('node:assert/strict');
const binary = path.resolve('target/debug/xtask');
const dir = fs.mkdtempSync(path.join(os.tmpdir(), 'discovery-ui-'));
function write(name, value) {
  const file = path.join(dir,name);fs.mkdirSync(path.dirname(file),{recursive:true});
  fs.writeFileSync(file,typeof value === 'string' ? value : JSON.stringify(value));
}
function run(action, input, ok=true) {
  const args=['task-artifact',action,'--task','UI'];
  if (input) { write('target/input.json',input);args.push('--input','target/input.json'); }
  const result=cp.spawnSync(binary,args,{cwd:dir,encoding:'utf8'});
  const data=JSON.parse(result.stdout);assert.equal(data.ok,ok,JSON.stringify(data));return data;
}
function localize(value) {
  if (Array.isArray(value)) return value.map(localize);
  if (value && typeof value === 'object') return Object.fromEntries(Object.entries(value).map(([k,v])=>[k,k==='references'?['implementation.rs']:k==='source'?'implementation.rs':localize(v)]));
  return value;
}
let blob;
const errors=[];
function open(page) {
  const virtualConsole=new VirtualConsole();virtualConsole.on('jsdomError',e=>errors.push(e.message));
  const dom=new JSDOM(fs.readFileSync(path.join(dir,'harness/task-artifacts/UI',page),'utf8'),{
    url:'https://offline-test.invalid/'+page,runScripts:'dangerously',virtualConsole,
    beforeParse(w) {
      w.Blob=Blob;w.URL.createObjectURL=b=>{blob=b;return 'blob:test';};w.URL.revokeObjectURL=()=>{};
      w.HTMLAnchorElement.prototype.click=function(){};
      Object.defineProperty(w.navigator,'clipboard',{value:{writeText:async()=>{}}});
    }
  });
  return dom;
}
function click(dom,selector) { const el=dom.window.document.querySelector(selector);assert.ok(el,selector);el.click(); }
async function exportAndImport(dom,expectedKey,expectedValue) {
  const doc=dom.window.document;doc.querySelector('#actor').value='UI integration test';
  click(dom,'#export-feedback');assert.ok(blob);
  const exported=JSON.parse(await blob.text());assert.equal(exported.responses[expectedKey],expectedValue);
  run('decide',exported);run('decide',exported,false); // actual export cannot replay over newer feedback
  dom.window.close();return exported;
}
(async()=>{
  try {
    write('harness/config.json',{});write('Cargo.toml','[workspace]\n');write('implementation.rs','// independently inspected fixture\n');
    const task=JSON.parse(fs.readFileSync('examples/task-record.json','utf8'));
    Object.assign(task,{id:'UI',schema_version:4,requirements:[],related_gaps:[],status:'partial'});
    write('harness/tasks/UI.json',task);
    run('start',localize(JSON.parse(fs.readFileSync('examples/task-artifact-before.json','utf8'))));
    const discovery=localize(JSON.parse(fs.readFileSync('examples/task-discovery.json','utf8')));
    discovery.blindspots[0].title='<img src=x onerror=alert(1)>';
    run('discover',discovery);
    let dom=open('blindspots.html');
    assert.equal(dom.window.document.querySelectorAll('#workspace img').length,0);
    click(dom,'#export-feedback');assert.match(dom.window.document.querySelector('#export-status').textContent,/name or role/);
    click(dom,'[data-key="blindspot:flat-data"][data-value=include]');
    assert.match(dom.window.document.querySelector('#decision-summary').textContent,/Use typed findings/);
    await exportAndImport(dom,'blindspot:flat-data','include');
    dom=open('interventions.html');click(dom,'[data-key="intervention:typed"][data-value=selected]');
    assert.match(dom.window.document.querySelector('#decision-summary').textContent,/Typed offline discovery tools: selected/);
    await exportAndImport(dom,'intervention:typed','selected');
    dom=open('interview.html');click(dom,'[data-step="2"]');
    dom.window.document.querySelector('#custom-answer').value='A failure-recovery simulator';click(dom,'[data-custom="interview:future"]');
    click(dom,'[data-step="3"]');assert.match(dom.window.document.querySelector('#workspace').textContent,/A failure-recovery simulator/);
    click(dom,'[data-step="2"]');assert.equal(dom.window.document.querySelector('#custom-answer').value,'A failure-recovery simulator');
    await exportAndImport(dom,'interview:future','A failure-recovery simulator');
    dom=open('reference.html');
    const linked=dom.window.document.querySelector('[data-linked="history"]');linked.dispatchEvent(new dom.window.Event('mouseenter'));
    assert.equal(dom.window.document.querySelectorAll('[data-linked="history"].linked-active').length,3);
    linked.dispatchEvent(new dom.window.Event('mouseleave'));
    assert.equal(dom.window.document.querySelectorAll('.linked-active').length,0);
    dom.window.document.querySelector('#correction-history').value='Preserve revisions, but do not claim authenticated chronology.';
    click(dom,'[data-correction="history"]');
    await exportAndImport(dom,'reference:history','Preserve revisions, but do not claim authenticated chronology.');
    dom=open('plan.html');
    assert.match(dom.window.document.querySelector('.plan-preview pre').textContent,/Vec<Blindspot>/);
    assert.match(dom.window.document.querySelector('#workspace').textContent,/A failure-recovery simulator/);
    click(dom,'[data-key="plan:model"][data-value=prose]');
    assert.match(dom.window.document.querySelector('.plan-preview pre').textContent,/blindspots: String/);
    click(dom,'#reset-draft');assert.match(dom.window.document.querySelector('.plan-preview pre').textContent,/Vec<Blindspot>/);
    click(dom,'[data-key="plan:model"][data-value=typed]');
    await exportAndImport(dom,'plan:model','typed');
    dom=open('plan.html');assert.equal(dom.window.document.querySelector('[data-key="plan:model"][data-value=typed]').getAttribute('aria-pressed'),'true');dom.window.close();
    const record=JSON.parse(fs.readFileSync(path.join(dir,'harness/task-artifacts/UI/record.json'),'utf8'));
    assert.equal(record.feedback.length,5);assert.equal(record.discoveries.length,1);
    assert.deepEqual(errors,[]);
    console.log('Five real discovery UIs passed: controls, summaries, interview editing, linked semantics, plan previews, escaped text, export → CLI import, stale rejection and reopened decisions.');
  } finally { fs.rmSync(dir,{recursive:true,force:true}); }
})().catch(e=>{console.error(e);process.exitCode=1;});
