// Optional Node check; normal Rust development has no JavaScript runtime dependency.
// Exercises the actual event handler against a stub DOM, not browser layout/CSP.
const fs = require('node:fs');
const vm = require('node:vm');
const assert = require('node:assert/strict');
let handler;
let selected = null;
const feedback = { textContent: '' };
const details = { open: false };
const box = {
  dataset: { answer: '1' },
  querySelector: selector => selector === 'input:checked' ? selected : selector === '.feedback' ? feedback : details
};
const button = {
  closest: selector => { assert.equal(selector, '.question'); return box; },
  addEventListener: (event, fn) => { assert.equal(event, 'click'); handler = fn; }
};
const context = {
  document: { querySelectorAll: selector => { assert.equal(selector, '.check'); return [button]; } }
};
vm.runInNewContext(fs.readFileSync('xtask/src/task_quiz.js', 'utf8'), context);
handler();
assert.equal(feedback.textContent, 'Choose an answer first.');
assert.equal(details.open, false);
selected = { value: '0' };
handler();
assert.equal(feedback.textContent, 'Not quite — review the explanation and try again.');
assert.equal(details.open, true);
selected = { value: '1' };
handler();
assert.equal(feedback.textContent, 'Correct — review the explanation and evidence.');
console.log('Quiz handler: unanswered, wrong answer, correct retry and explanation disclosure passed.');
