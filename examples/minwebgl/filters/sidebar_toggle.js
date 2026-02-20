// Sidebar Toggle Controller

let sidebarVisible = true;

export function setupSidebarToggle() {
  const sidebar = document.getElementById('sidebar');
  const toggle = document.getElementById('sidebar-toggle');

  if (!sidebar || !toggle) {
    console.warn('Sidebar or toggle button not found');
    return;
  }

  toggle.addEventListener('click', () => {
    sidebarVisible = !sidebarVisible;

    if (sidebarVisible) {
      sidebar.classList.remove('hidden');
      toggle.classList.remove('sidebar-hidden');
    } else {
      sidebar.classList.add('hidden');
      toggle.classList.add('sidebar-hidden');
    }
  });

  console.log('🎛️ Sidebar toggle initialized');
}
