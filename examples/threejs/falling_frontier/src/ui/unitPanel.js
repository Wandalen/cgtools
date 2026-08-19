// Pushes a selected object's userData into the right-hand tactical detail panel.
export function selectUnit(obj) {
    const data = obj.userData;

    document.getElementById('unit-name').innerText = data.name || 'UNKNOWN UNIT';
    document.getElementById('unit-commander').innerText = data.commander || 'UNASSIGNED';
    document.getElementById('unit-type-tag').innerText = `${data.type || 'OBJECT'} CLASS`;

    const panel = document.getElementById('unit-panel');
    panel.classList.add('ui-panel-glow');
    setTimeout(() => panel.classList.remove('ui-panel-glow'), 600);
}
