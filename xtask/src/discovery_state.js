// Pure state operations are shared by the actual UI and reproducible interaction checks.
const DiscoveryState = (() => {
  function effective(data, edits, key) {
    if (Object.hasOwn(edits, key)) return edits[key];
    if (Object.hasOwn(data.responses, key)) return data.responses[key];
    if (key.startsWith('interview:')) {
      return data.content.interview.questions.find(q => key === 'interview:' + q.id)?.settled?.value ?? '';
    }
    return '';
  }
  function planOption(data, edits, choice) {
    const id = effective(data, edits, 'plan:' + choice.id) || choice.recommended;
    return choice.options.find(o => o.id === id);
  }
  function progress(data, edits) {
    const qs = data.content.interview.questions;
    return { answered: qs.filter(q => effective(data, edits, 'interview:' + q.id).trim()).length, total: qs.length };
  }
  function summary(data, edits) {
    const d = data.content, lines = [];
    const add = (key, label, format = x => x) => {
      const value = effective(data, edits, key);
      if (value) lines.push(label + ': ' + format(value));
    };
    if (data.page === 'blindspots.html') d.blindspots.forEach(b => add('blindspot:' + b.id, b.title, v => v === 'include' ? b.constraint : v));
    if (data.page === 'interventions.html') d.interventions.forEach(i => add('intervention:' + i.id, i.title));
    if (data.page === 'interview.html') d.interview.questions.forEach(q => add('interview:' + q.id, q.prompt));
    if (data.page === 'reference.html') d.reference.mappings.forEach(m => add('reference:' + m.id, m.behavior));
    if (data.page === 'plan.html') d.plan.choices.forEach(c => add('plan:' + c.id, c.title, id => c.options.find(o => o.id === id)?.label || id));
    return lines.length ? lines.join('\n\n') : 'No explicit decisions yet. Explore the options above.';
  }
  function implementationBrief(data, edits) {
    const sections = [['blindspots.html','Implementation constraints'],['interventions.html','Interventions'],['interview.html','Requirements and answers'],['reference.html','Reference review'],['plan.html','Plan selections']];
    return sections.map(([page,title]) => title + '\n' + summary({...data,page},edits)).join('\n\n');
  }
  function payload(data, edits, actor) {
    if (!actor.trim()) throw new Error('Enter your name or role before exporting.');
    if (!Object.keys(edits).length) throw new Error('Make an explicit choice or correction before exporting.');
    return { schema_version: 1, task: data.task, discovery_sha256: data.discovery_sha256,
      feedback_base_sha256: data.feedback_base_sha256, actor: actor.trim(), responses: { ...edits } };
  }
  return { effective, planOption, progress, summary, implementationBrief, payload };
})();
if (typeof module !== 'undefined') module.exports = DiscoveryState;
