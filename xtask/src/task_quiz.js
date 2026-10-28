// Offline comprehension feedback. It never writes task state or grants approval.
function answerFeedback(selected, expected) {
  if (selected === null) return 'Choose an answer first.';
  return selected === expected
    ? 'Correct — review the explanation and evidence.'
    : 'Not quite — review the explanation and try again.';
}
document.querySelectorAll('.check').forEach(button => {
  button.addEventListener('click', () => {
    const box = button.closest('.question');
    const chosen = box.querySelector('input:checked');
    box.querySelector('.feedback').textContent = answerFeedback(chosen ? chosen.value : null, box.dataset.answer);
    if (chosen) box.querySelector('details').open = true;
  });
});
