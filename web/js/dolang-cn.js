// ── SCROLL PROGRESS ──────────────────────
const progress = document.getElementById('progress');
window.addEventListener('scroll', () => {
    const pct = window.scrollY / (document.documentElement.scrollHeight - window.innerHeight) * 100;
    progress.style.width = Math.min(pct, 100) + '%';
}, { passive: true });

// ── NAV SCROLL STATE ──────────────────────
const nav = document.getElementById('nav');
window.addEventListener('scroll', () => {
    nav.classList.toggle('scrolled', window.scrollY > 60);
}, { passive: true });

// ── PARALLAX ─────────────────────────────
const heroRing = document.getElementById('heroRing');
const heroCode = document.getElementById('heroCode');
window.addEventListener('scroll', () => {
    const y = window.scrollY;
    const isMobile = window.innerWidth <= 1240;
    if (heroRing) heroRing.style.transform = `translateY(calc(-50% + ${y * 0.22}px))`;
    if (heroCode) {
        if (!isMobile) {
            heroCode.style.transform = `translateY(${y * 0.13}px) rotate(${y * 0.005}deg)`;
        } else {
            heroCode.style.transform = `none`;
        }
    }
}, { passive: true });

// ── INTERSECTION OBSERVER ─────────────────
const io = new IntersectionObserver((entries) => {
    entries.forEach(e => {
        if (e.isIntersecting) {
            e.target.classList.add('visible');

            const statNum = e.target.classList.contains('stat-cell')
                ? e.target.querySelector('.stat-num')
                : e.target;

            if (statNum && statNum.dataset && statNum.dataset.target !== undefined) {
                animateCount(statNum);
            }
            io.unobserve(e.target);
        }
    });
}, { threshold: 0.14, rootMargin: '0px 0px -60px 0px' });

document.querySelectorAll('.reveal, .feat-card, .stat-cell, .install-box').forEach(el => io.observe(el));

// Showcase window reveal
const showcaseWin = document.getElementById('showcaseWin');
if (showcaseWin) io.observe(showcaseWin);

// ── COUNTER ANIMATION ─────────────────────
function easeOutCubic(t) { return 1 - Math.pow(1 - t, 3); }

function animateCount(el) {
    const target = parseInt(el.dataset.target);
    const suffix = el.dataset.suffix || '';
    const duration = 1700;
    const start = Date.now();
    (function tick() {
        const p = Math.min((Date.now() - start) / duration, 1);
        el.textContent = Math.floor(easeOutCubic(p) * target) + suffix;
        if (p < 1) requestAnimationFrame(tick);
    })();
}

// ── CODE EXAMPLES ─────────────────────────
const codeExamples = {
    basics: {
        fname: 'basics.dal',
        lines: [
            '<span class="tw-cm">// Declare and print a variable</span>',
            '<span class="tw-kw">$</span> city <span class="tw-kw">=</span> <span class="tw-str">"London"</span>;',
            '<span class="tw-kw">$&gt;&gt;</span> city;',
            '<span class="tw-cm">London</span>',
            '',
            '<span class="tw-cm">// Integer arithmetic</span>',
            '<span class="tw-kw">$</span> x <span class="tw-kw">=</span> <span class="tw-num">42</span>;',
            '<span class="tw-kw">$&gt;&gt;</span> x;',
            '<span class="tw-cm">42</span>',
            '',
            '<span class="tw-cm">// Float</span>',
            '<span class="tw-kw">$</span> pi <span class="tw-kw">=</span> <span class="tw-num">3.14</span>;',
            '<span class="tw-kw">$&gt;&gt;</span> pi;',
            '<span class="tw-cm">3.14</span>',
        ]
    },
    functions: {
        fname: 'functions.dal',
        lines: [
            '<span class="tw-cm">// Define a function</span>',
            '<span class="tw-kw">$fn</span> add(a, b) {',
            '    <span class="tw-kw">$#</span> a + b;',
            '}',
            '',
            '<span class="tw-cm">// Call a function</span>',
            '<span class="tw-kw">$</span> sum <span class="tw-kw">=</span> add(<span class="tw-num">1</span>, <span class="tw-num">2</span>);',
            '<span class="tw-kw">$&gt;&gt;</span> sum;',
            '<span class="tw-cm">3</span>',
        ]
    },
    collections: {
        fname: 'collections.dal',
        lines: [
            '<span class="tw-cm">// List implementation</span>',
            '<span class="tw-kw">$</span> arr <span class="tw-kw">=</span> [<span class="tw-num">1</span>, <span class="tw-num">2</span>, <span class="tw-num">3</span>];',
            '<span class="tw-kw">$&gt;&gt;</span> arr[<span class="tw-num">0</span>];',
            '<span class="tw-cm">1</span>',
            '',
            '<span class="tw-cm">// Map implementation</span>',
            '<span class="tw-kw">$</span> user <span class="tw-kw">=</span> {<span class="tw-str">"name"</span>: <span class="tw-str">"Tom"</span>};',
            '<span class="tw-kw">$&gt;&gt;</span> user[<span class="tw-str">"name"</span>];',
            '<span class="tw-cm">Tom</span>',
        ]
    },
    const: {
        fname: 'constants.dal',
        lines: [
            '<span class="tw-cm">// Constants are immutable</span>',
            '<span class="tw-kw">$@</span> PI <span class="tw-kw">=</span> <span class="tw-num">3.14</span>;',
            '<span class="tw-kw">$&gt;&gt;</span> PI;',
            '<span class="tw-cm">3.14</span>',
            '',
            '<span class="tw-cm">// Attempt to reassign →</span>',
            'PI <span class="tw-kw">=</span> <span class="tw-num">99</span>;',
            '<span class="tw-cm">[ERROR] cannot reassign constant \'PI\'</span>',
        ]
    }
};

let currentTab = 'basics';
let twTimer = null;

function typeLines(tab) {
    clearTimeout(twTimer);
    const { lines, fname } = codeExamples[tab];
    const out = document.getElementById('typewriterOut');
    const fnEl = document.getElementById('showcaseFname');
    if (fnEl) fnEl.textContent = fname;
    out.innerHTML = '<span class="tw-cursor"></span>';
    let i = 0;

    function next() {
        if (i >= lines.length) return;
        const div = document.createElement('div');
        div.innerHTML = lines[i] || '&nbsp;';
        div.style.opacity = '0';
        out.insertBefore(div, out.querySelector('.tw-cursor'));
        requestAnimationFrame(() => {
            div.style.transition = 'opacity 0.18s ease';
            div.style.opacity = '1';
        });
        i++;
        twTimer = setTimeout(next, lines[i - 1] === '' ? 70 : 110);
    }
    next();
}

// Tab buttons
document.querySelectorAll('.tab-btn').forEach(btn => {
    btn.addEventListener('click', () => {
        document.querySelectorAll('.tab-btn').forEach(b => b.classList.remove('active'));
        btn.classList.add('active');
        currentTab = btn.dataset.tab;
        typeLines(currentTab);
    });
});

// Trigger typewriter when showcase enters view
const showcaseSection = document.getElementById('showcase');
const showcaseIo = new IntersectionObserver((entries) => {
    entries.forEach(e => {
        if (e.isIntersecting) {
            document.getElementById('showcaseWin').classList.add('visible');
            typeLines(currentTab);
            showcaseIo.unobserve(e.target);
        }
    });
}, { threshold: 0.25 });
if (showcaseSection) showcaseIo.observe(showcaseSection);

// ── COPY INSTALL COMMAND ──────────────────
function copyCmd(el, text) {
    navigator.clipboard.writeText(text).then(() => {
        const icon = el.querySelector('.copy-icon');
        if (icon) {
            icon.textContent = '✓ done';
            icon.style.color = 'var(--sage)';
            setTimeout(() => {
                icon.textContent = 'copy';
                icon.style.color = '';
            }, 1800);
        }
    });
}

// ── HERO REPL ──────────────────────────────
const heroReplInput = document.getElementById('heroReplInput');
const heroRunBtn = document.getElementById('heroRunBtn');
const heroReplOutput = document.getElementById('heroReplOutput');

// ── HERO SYNTAX HIGHLIGHTING ──────────────
const heroReplHighlight = document.getElementById('heroReplHighlight');

function highlightDolangCode(code) {
    if (!code) return '';
    let escaped = code.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;");

    const tokenRegex = /(&quot;.*?&quot;|&#39;.*?&#39;|".*?"|'.*?'|\/\/.*|\$@|\$fn|\$&gt;&gt;|\$#|\$|=|\b\d+(?:\.\d+)?\b|[a-zA-Z_]\w*(?=\())/g;

    return escaped.replace(tokenRegex, (match) => {
        if (match.startsWith('//')) {
            return '<span class="tw-cm" style="color:var(--iron-light); font-style:italic;">' + match + '</span>';
        } else if (match.startsWith('"') || match.startsWith("'") || match.startsWith('&quot;') || match.startsWith('&#39;')) {
            return '<span class="tw-str" style="color:#C67A5D;">' + match + '</span>';
        } else if (match === '$@' || match === '$fn' || match === '$&gt;&gt;' || match === '$#' || match === '$' || match === '=') {
            return '<span class="tw-kw" style="color:var(--rust); font-weight:bold;">' + match + '</span>';
        } else if (/^\d/.test(match)) {
            return '<span class="tw-num" style="color:#D97757;">' + match + '</span>';
        } else {
            return '<span class="tw-fn" style="color:var(--gold-deep);">' + match + '</span>';
        }
    });
}

if (heroReplInput && heroReplHighlight) {
    const updateSyntax = () => {
        let val = heroReplInput.value;
        if (val[val.length - 1] === '\n') {
            val += ' ';
        }
        heroReplHighlight.innerHTML = highlightDolangCode(val);
    };

    heroReplInput.addEventListener('input', updateSyntax);
    heroReplInput.addEventListener('scroll', () => {
        heroReplHighlight.scrollTop = heroReplInput.scrollTop;
        heroReplHighlight.scrollLeft = heroReplInput.scrollLeft;
    });
    // Initial state
    updateSyntax();
}

if (heroRunBtn && heroReplInput) {
    heroRunBtn.addEventListener('click', (e) => {
        e.preventDefault();
        const val = heroReplInput.value.trim();
        if (!val) {
            heroReplOutput.innerHTML = '';
            return;
        }

        let output = "The script was parsed successfully.";

        if (val === 'clear') {
            heroReplInput.value = '';
            heroReplOutput.innerHTML = '';
            return;
        } else if (val.includes('"')) {
            const parts = val.split('"');
            output = parts.length > 1 ? parts[1] : val;
        } else if (val.includes('+')) {
            output = "Dolang evaluates: result";
        }

        // Add a small fade effect to        // 添加淡入效果模拟处理
        heroReplOutput.style.opacity = '0';
        setTimeout(() => {
            heroReplOutput.innerHTML = `输出: ${output}`;
            heroReplOutput.style.transition = 'opacity 0.3s ease';
            heroReplOutput.style.opacity = '1';
        }, 150);
    });
}

// ── OS TABS ───────────────────────────────
document.querySelectorAll('.os-tab').forEach(tab => {
    tab.addEventListener('click', (e) => {
        const os = e.target.dataset.os;
        document.querySelectorAll('.os-tab').forEach(t => t.classList.remove('active'));
        e.target.classList.add('active');

        const macSteps = document.getElementById('installStepsMac');
        const winSteps = document.getElementById('installStepsWin');

        if (os === 'mac') {
            if (macSteps) macSteps.style.display = 'block';
            if (winSteps) winSteps.style.display = 'none';
        } else {
            if (macSteps) macSteps.style.display = 'none';
            if (winSteps) winSteps.style.display = 'block';
        }
    });
});

// ── LANGUAGE SWITCHER ─────────────────────
const navLang = document.querySelector('.nav-lang');
const langCurrent = document.querySelector('.lang-current');
if (navLang && langCurrent) {
    langCurrent.addEventListener('click', (e) => {
        e.preventDefault();
        navLang.classList.toggle('open');
    });

    document.addEventListener('click', (e) => {
        if (!e.target.closest('.nav-lang')) {
            navLang.classList.remove('open');
        }
    });
}