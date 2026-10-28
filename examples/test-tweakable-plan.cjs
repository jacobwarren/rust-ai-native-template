// Optional pattern: adapt selectors, choices and independent expectations to your page.
const {JSDOM}=require('jsdom');
const fs=require('node:fs'),crypto=require('node:crypto'),assert=require('node:assert/strict');
const [page,report]=process.argv.slice(2);if(!page||!report)throw Error('Provide HTML and output JSON paths');
const bytes=fs.readFileSync(page);const dom=new JSDOM(bytes.toString(),{runScripts:'dangerously'});
const d=dom.window.document;const cases=[];
for(const [choice,after,response] of [
 ['presence','Eight files can still be produced retrospectively at completion.','Use presence checks only; accept missing during-work evidence.'],
 ['stages','Final publication waits for a separate implementation note revision.','Use discovery, implementation and final checkpoints; retain free HTML composition.']
]){
 const before=d.querySelector('#outcome').textContent;
 d.querySelector('#mode').value=choice;d.querySelector('#mode').dispatchEvent(new dom.window.Event('change'));
 assert.equal(d.querySelector('#outcome').textContent,after);
 assert.equal(d.querySelector('#response').value,response);
 assert.notEqual(before,after);cases.push({choice,before,after,response});
}
fs.writeFileSync(report,JSON.stringify({schema_version:1,plan_sha256:crypto.createHash('sha256').update(bytes).digest('hex'),test_command:'node examples/test-tweakable-plan.cjs '+page+' '+report,cases},null,2)+'\n');
dom.window.close();console.log('Plan handlers and responses passed; browser visual review remains separate.');
