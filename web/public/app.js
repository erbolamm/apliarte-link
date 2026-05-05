// app.js — Interactivity for ApliArte Link landing
(function () {
  'use strict';

  // ===== Language toggle =====
  const STORAGE_KEY = 'apliarte-link-lang';
  const langToggle = document.getElementById('langToggle');
  const langActiveEl = langToggle?.querySelector('.lang-active');
  const langOtherEl = langToggle?.querySelector('.lang-other');

  function getInitialLang() {
    const saved = localStorage.getItem(STORAGE_KEY);
    if (saved === 'es' || saved === 'en') return saved;
    const browser = (navigator.language || 'es').slice(0, 2).toLowerCase();
    return browser === 'en' ? 'en' : 'es';
  }

  function applyLang(lang) {
    if (!I18N || !I18N[lang]) return;
    document.documentElement.lang = lang;
    document.querySelectorAll('[data-i18n]').forEach((el) => {
      const key = el.getAttribute('data-i18n');
      const value = I18N[lang][key];
      if (value !== undefined) el.innerHTML = value;
    });
    if (langActiveEl) langActiveEl.textContent = lang.toUpperCase();
    if (langOtherEl) langOtherEl.textContent = (lang === 'es' ? 'EN' : 'ES');
    localStorage.setItem(STORAGE_KEY, lang);
  }

  let currentLang = getInitialLang();
  applyLang(currentLang);

  langToggle?.addEventListener('click', () => {
    currentLang = currentLang === 'es' ? 'en' : 'es';
    applyLang(currentLang);
  });

  // ===== Install tabs =====
  const tabBtns = document.querySelectorAll('.tab-btn');
  const panels = document.querySelectorAll('.install-panel');
  tabBtns.forEach((btn) => {
    btn.addEventListener('click', () => {
      const target = btn.dataset.tab;
      tabBtns.forEach((b) => b.classList.toggle('active', b === btn));
      panels.forEach((p) => p.classList.toggle('active', p.dataset.panel === target));
    });
  });

  // ===== Copy buttons =====
  document.querySelectorAll('.copy-btn').forEach((btn) => {
    btn.addEventListener('click', async (e) => {
      e.preventDefault();
      const block = btn.closest('.code-block');
      const code = block?.querySelector('code')?.textContent || '';
      try {
        await navigator.clipboard.writeText(code);
        btn.classList.add('copied');
        const original = btn.textContent;
        btn.textContent = '✓';
        setTimeout(() => {
          btn.classList.remove('copied');
          btn.textContent = original;
        }, 1500);
      } catch (err) {
        console.error('Copy failed:', err);
      }
    });
  });

  // ===== Smooth scroll offset for sticky header =====
  document.querySelectorAll('a[href^="#"]').forEach((a) => {
    a.addEventListener('click', (e) => {
      const id = a.getAttribute('href');
      if (id === '#' || id.length < 2) return;
      const target = document.querySelector(id);
      if (!target) return;
      e.preventDefault();
      const offset = 80;
      const top = target.getBoundingClientRect().top + window.pageYOffset - offset;
      window.scrollTo({ top, behavior: 'smooth' });
    });
  });

  // ===== Fade-in on scroll =====
  if ('IntersectionObserver' in window) {
    const observer = new IntersectionObserver(
      (entries) => {
        entries.forEach((entry) => {
          if (entry.isIntersecting) {
            entry.target.style.opacity = '1';
            entry.target.style.transform = 'translateY(0)';
            observer.unobserve(entry.target);
          }
        });
      },
      { threshold: 0.1, rootMargin: '0px 0px -40px 0px' }
    );
    document.querySelectorAll('.feature, .roadmap-item, .plan-card, .support-card, .install-card').forEach((el) => {
      el.style.opacity = '0';
      el.style.transform = 'translateY(20px)';
      el.style.transition = 'opacity 0.6s ease, transform 0.6s ease';
      observer.observe(el);
    });
  }
})();
