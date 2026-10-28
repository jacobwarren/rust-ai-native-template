(() => {
  'use strict';
  const data = JSON.parse(document.getElementById('discovery-data').textContent);
  const d = data.content, S = DiscoveryState;
  const root = document.getElementById('workspace');
  const status = document.getElementById('export-status');
  const draftKey = ['task-discovery', data.task, data.page, data.discovery_sha256, data.feedback_base_sha256].join(':');
  let edits = {}, current = 0;
  try { edits = JSON.parse(sessionStorage.getItem(draftKey) || '{}'); } catch (_) { /* Storage may be disabled for local files. */ }
  const esc = value => String(value).replace(/[&<>"']/g, c => ({'&':'&amp;','<':'&lt;','>':'&gt;','"':'&quot;',"'":'&#39;'}[c]));
  const value = key => S.effective(data, edits, key);
  const refs = list => '<ul class=refs>' + list.map(r => '<li><a rel=noreferrer href="' + esc(r.startsWith('https://') ? r : '../../../' + r.split('/').map(encodeURIComponent).join('/')) + '">' + esc(r) + '</a></li>').join('') + '</ul>';
  const button = (label, key, val, selected) => '<button type=button class="choice' + (selected ? ' selected' : '') + '" data-key="' + esc(key) + '" data-value="' + esc(val) + '" aria-pressed="' + !!selected + '">' + esc(label) + '</button>';
  function changed() {
    try { sessionStorage.setItem(draftKey, JSON.stringify(edits)); } catch (_) { /* Export still works without storage. */ }
    document.getElementById('decision-summary').textContent = S.summary(data, edits);
  }
  function choose(key, val) { edits[key] = val; changed(); render(); }
  function blindspots() {
    if (!d.blindspots.length) return '<div class=block><h2>No material blindspots found in the inspected scope</h2><p>' + esc(d.no_blindspots_reason) + '</p>' + refs(d.references) + '</div>';
    return '<p class=intro>Understand what can go wrong, inspect the evidence, and assemble the constraints that should guide implementation.</p>' + d.blindspots.map((b, i) => '<article class="finding block"><span class=eyebrow>' + esc(b.category) + ' · ' + String(i+1).padStart(2,'0') + '</span><h2>' + esc(b.title) + '</h2><p>' + esc(b.observation) + '</p><div class=consequence><h3>Why it matters</h3><p>' + esc(b.consequence) + '</p></div><div class=constraint><h3>Implementation constraint</h3><p>' + esc(b.constraint) + '</p></div>' + refs(b.references) + '<div class=actions>' + [['Include constraint','include'],['Investigate first','investigate'],['Exclude from scope','exclude']].map(([t,v]) => button(t,'blindspot:'+b.id,v,value('blindspot:'+b.id)===v)).join('') + '</div></article>').join('');
  }
  function interventions() {
    const sorted = [...d.interventions].sort((a,b) => a.effort-b.effort);
    return '<p class=intro>Compare approaches grounded in the inspected code. Effort is relative; impact remains a hypothesis until measured.</p><div class=effort-scale><span>Smaller change</span><div>' + sorted.map(i => '<a class=effort-dot href="#intervention-' + esc(i.id) + '">' + i.effort + '</a>').join('') + '</div><span>Larger change</span></div>' + sorted.map(i => '<article id="intervention-' + esc(i.id) + '" class="block intervention"><span class=eyebrow>EFFORT ' + i.effort + ' / 5</span><h2>' + esc(i.title) + '</h2><p>' + esc(i.approach) + '</p><div class=compare-grid><div><h3>Expected benefit</h3><p>' + esc(i.benefit) + '</p></div><div><h3>Tradeoff</h3><p>' + esc(i.tradeoff) + '</p></div></div>' + refs(i.references) + '<div class=actions>' + button('This resonates','intervention:'+i.id,'selected',value('intervention:'+i.id)==='selected') + button('Leave aside','intervention:'+i.id,'rejected',value('intervention:'+i.id)==='rejected') + '</div></article>').join('');
  }
  function interview() {
    const qs=d.interview.questions, p=S.progress(data,edits);
    if (!qs.length) return '<div class=block><h2>No unresolved questions</h2><p>' + esc(d.interview.no_questions_reason) + '</p>' + refs(d.references) + '</div>';
    const rail='<div class=interview-rail><p>' + p.answered + ' of ' + p.total + ' answered or settled</p>' + qs.map((q,i) => '<button type=button data-step="' + i + '" class="step' + (i===current?' selected':'') + '">' + (value('interview:'+q.id)?'✓ ':'○ ') + esc(q.prompt) + '</button>').join('') + '<button type=button data-step="' + qs.length + '">Review decisions →</button></div>';
    if (current>=qs.length) return '<div class=interview-layout>' + rail + '<article class=block><h2>Your implementation brief</h2>' + qs.map((q,i)=>'<div class=answer-row><h3>'+esc(q.prompt)+'</h3><p>'+esc(value('interview:'+q.id)||'Unanswered')+'</p><button class=secondary type=button data-step="'+i+'">Edit answer</button></div>').join('') + '</article></div>';
    const q=qs[current], key='interview:'+q.id;
    return '<div class=interview-layout>' + rail + '<article class=block><span class=eyebrow>QUESTION ' + (current+1) + ' / ' + qs.length + (q.blocking?' · REQUIRED BEFORE COMPLETION':' · EXPLORATORY') + '</span><h2>' + esc(q.prompt) + '</h2><p>' + esc(q.why) + '</p>' + (q.settled?'<p class=settled>Existing answer: '+esc(q.settled.value)+'<br>Basis: '+esc(q.settled.basis)+'</p>':'') + q.options.map(o => '<button class="answer-choice'+(value(key)===o.label?' selected':'')+'" type=button data-key="'+esc(key)+'" data-value="'+esc(o.label)+'"><strong>'+esc(o.label)+'</strong><span>'+esc(o.consequence)+'</span></button>').join('') + '<label class=custom-answer>Your own answer<textarea id=custom-answer rows=3>'+esc(value(key))+'</textarea></label><button type=button data-custom="'+esc(key)+'">Use this answer</button><div class=interview-nav><button type=button class=secondary data-step="'+Math.max(0,current-1)+'"'+(current===0?' disabled':'')+'>Previous</button><button type=button data-step="'+(current+1)+'">'+(current===qs.length-1?'Review decisions':'Next question')+'</button></div></article></div>';
  }
  function reference() {
    const r=d.reference;
    if (!r.mappings.length) return '<div class=block><h2>No suitable reference identified</h2><p>'+esc(r.missing_reason)+'</p><p>This is a documented gap, not an invented source comparison.</p>'+refs(d.references)+'</div>';
    return '<div class=reference-meta><strong>'+esc(r.name)+'</strong><span>Revision: '+esc(r.revision)+'</span><span>Source: '+esc(r.source)+'</span><span>Target: '+esc(r.target)+'</span></div><p class=intro>Focus or hover a source, target, or behavior note to highlight its matching semantics. Correct the interpretation before building on it.</p>'+r.mappings.map(m=>'<article class="block mapping" data-mapping="'+esc(m.id)+'"><h2>'+esc(m.behavior)+'</h2><div class=compare-grid><div tabindex=0 data-linked="'+esc(m.id)+'"><span class=eyebrow>REFERENCE</span><pre>'+esc(m.source_excerpt)+'</pre></div><div tabindex=0 data-linked="'+esc(m.id)+'"><span class=eyebrow>PROPOSED TARGET</span><pre>'+esc(m.target_excerpt)+'</pre></div></div><div tabindex=0 data-linked="'+esc(m.id)+'" class=semantic-note><h3>Difference / translation concern</h3><p>'+esc(m.difference)+'</p><h3>Independent expected behavior</h3><p>'+esc(m.expected)+'</p></div><div class=actions>'+button('Interpretation matches','reference:'+m.id,'confirmed',value('reference:'+m.id)==='confirmed')+'</div><label class=custom-answer>Correction<textarea rows=2 id="correction-'+esc(m.id)+'">'+esc(value('reference:'+m.id)==='confirmed'?'':value('reference:'+m.id))+'</textarea></label><button class=secondary type=button data-correction="'+esc(m.id)+'">Record correction</button></article>').join('')+refs([r.source]);
  }
  function plan() {
    return '<p class=intro>Consequential choices come first. Toggle an alternative to see its interface or behavior preview. A recommendation is not a recorded human decision.</p>'+d.plan.choices.map(c=>{
      const selected=value('plan:'+c.id), o=S.planOption(data,edits,c);
      return '<article class="block plan-choice"><span class=eyebrow>DECISION TO REVIEW</span><h2>'+esc(c.title)+'</h2><p>'+esc(c.why)+'</p><div class=actions>'+c.options.map(o=>button(o.label+(o.id===c.recommended?' · recommended':''),'plan:'+c.id,o.id,selected===o.id)).join('')+'</div><div class=plan-preview><span class=eyebrow>'+(selected?'YOUR PROPOSED CHOICE':'RECOMMENDATION PREVIEW · NO EXPLICIT SELECTION')+'</span><h3>'+esc(o.label)+'</h3><p>'+esc(o.tradeoff)+'</p><pre>'+esc(o.preview)+'</pre></div></article>';
    }).join('')+'<section class=block><h2>Decision-informed implementation brief</h2><p>Imported responses and sourced existing answers, plus current plan proposals. Reconcile these decisions into the execution steps before implementation.</p><pre>'+esc(S.implementationBrief(data,edits))+'</pre></section><section class=block><h2>Execution order</h2><ol>'+d.plan.steps.map(s=>'<li>'+esc(s)+'</li>').join('')+'</ol></section><details class="block mechanical"><summary>Mechanical work and housekeeping ('+d.plan.mechanical.length+')</summary><ul>'+d.plan.mechanical.map(s=>'<li>'+esc(s)+'</li>').join('')+'</ul></details>';
  }
  function render() {
    root.innerHTML=({'blindspots.html':blindspots,'interventions.html':interventions,'interview.html':interview,'reference.html':reference,'plan.html':plan}[data.page])();
    root.querySelectorAll('[data-key]').forEach(el=>el.addEventListener('click',()=>choose(el.dataset.key,el.dataset.value)));
    root.querySelectorAll('[data-step]').forEach(el=>el.addEventListener('click',()=>{current=Number(el.dataset.step);render();root.querySelector('h2')?.setAttribute('tabindex','-1');root.querySelector('h2')?.focus();}));
    root.querySelectorAll('[data-custom]').forEach(el=>el.addEventListener('click',()=>{const v=root.querySelector('#custom-answer').value.trim();if(v)choose(el.dataset.custom,v);}));
    root.querySelectorAll('[data-correction]').forEach(el=>el.addEventListener('click',()=>{const v=document.getElementById('correction-'+el.dataset.correction).value.trim();if(v)choose('reference:'+el.dataset.correction,v);}));
    root.querySelectorAll('[data-linked]').forEach(el=>{
      const highlight=on=>root.querySelectorAll('[data-linked]').forEach(x=>{if(x.dataset.linked===el.dataset.linked)x.classList.toggle('linked-active',on);});
      el.addEventListener('mouseenter',()=>highlight(true));el.addEventListener('mouseleave',()=>highlight(false));
      el.addEventListener('focus',()=>highlight(true));el.addEventListener('blur',()=>highlight(false));
    });
    changed();
  }
  document.getElementById('export-feedback').addEventListener('click',()=>{
    try {
      const payload=S.payload(data,edits,document.getElementById('actor').value);
      const url=URL.createObjectURL(new Blob([JSON.stringify(payload,null,2)+'\n'],{type:'application/json'}));
      const a=document.createElement('a');a.href=url;a.download=data.task+'-'+data.page.replace('.html','')+'-decisions.json';a.click();setTimeout(()=>URL.revokeObjectURL(url),1000);
      status.textContent='Exported proposals. Import the JSON to record them; repository files have not changed.';
    } catch(e) { status.textContent=e.message; }
  });
  document.getElementById('copy-summary').addEventListener('click',async()=>{
    const text=S.summary(data,edits);
    try { await navigator.clipboard.writeText(text);status.textContent='Summary copied.'; }
    catch(_) { status.textContent='Clipboard unavailable. Select and copy the summary text above.'; }
  });
  document.getElementById('reset-draft').addEventListener('click',()=>{edits={};render();status.textContent='Draft changes discarded. Imported decisions are unchanged.';});
  render();
})();
