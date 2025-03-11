// Populate the sidebar
//
// This is a script, and not included directly in the page, to control the total size of the book.
// The TOC contains an entry for each page, so if each page includes a copy of the TOC,
// the total size of the page becomes O(n**2).
class MDBookSidebarScrollbox extends HTMLElement {
    constructor() {
        super();
    }
    connectedCallback() {
        this.innerHTML = '<ol class="chapter"><li class="chapter-item expanded affix "><a href="index.html">Introduction</a></li><li class="chapter-item expanded "><a href="overview/index.html"><strong aria-hidden="true">1.</strong> Overview</a></li><li><ol class="section"><li class="chapter-item expanded "><a href="overview/terminology.html"><strong aria-hidden="true">1.1.</strong> Terminology</a></li><li class="chapter-item expanded "><a href="overview/features.html"><strong aria-hidden="true">1.2.</strong> Features</a></li></ol></li><li class="chapter-item expanded "><a href="quick/index.html"><strong aria-hidden="true">2.</strong> Quick start</a></li><li><ol class="section"><li class="chapter-item expanded "><a href="quick/codespace.html"><strong aria-hidden="true">2.1.</strong> GitHub Codespace tips</a></li></ol></li><li class="chapter-item expanded "><a href="applet/index.html"><strong aria-hidden="true">3.</strong> Applet user guide</a></li><li><ol class="section"><li class="chapter-item expanded "><a href="applet/create.html"><strong aria-hidden="true">3.1.</strong> Create a new applet</a></li><li class="chapter-item expanded "><a href="applet/run.html"><strong aria-hidden="true">3.2.</strong> Run an applet</a></li><li class="chapter-item expanded "><a href="applet/api.html"><strong aria-hidden="true">3.3.</strong> API</a></li><li class="chapter-item expanded "><a href="applet/prelude/index.html"><strong aria-hidden="true">3.4.</strong> Prelude</a></li><li><ol class="section"><li class="chapter-item expanded "><a href="applet/prelude/led.html"><strong aria-hidden="true">3.4.1.</strong> LEDs</a></li><li class="chapter-item expanded "><a href="applet/prelude/button.html"><strong aria-hidden="true">3.4.2.</strong> Buttons</a></li><li class="chapter-item expanded "><a href="applet/prelude/timer.html"><strong aria-hidden="true">3.4.3.</strong> Timers</a></li><li class="chapter-item expanded "><a href="applet/prelude/usb.html"><strong aria-hidden="true">3.4.4.</strong> USB</a></li><li class="chapter-item expanded "><a href="applet/prelude/uart.html"><strong aria-hidden="true">3.4.5.</strong> UART</a></li><li class="chapter-item expanded "><a href="applet/prelude/rpc.html"><strong aria-hidden="true">3.4.6.</strong> RPC</a></li><li class="chapter-item expanded "><a href="applet/prelude/store.html"><strong aria-hidden="true">3.4.7.</strong> Storage</a></li></ol></li><li class="chapter-item expanded "><a href="applet/exercises/index.html"><strong aria-hidden="true">3.5.</strong> Exercises</a></li><li class="chapter-item expanded "><a href="applet/examples.html"><strong aria-hidden="true">3.6.</strong> Examples</a></li></ol></li><li class="chapter-item expanded "><a href="runner/index.html"><strong aria-hidden="true">4.</strong> Runner user guide</a></li><li><ol class="section"><li class="chapter-item expanded "><a href="runner/api.html"><strong aria-hidden="true">4.1.</strong> API</a></li></ol></li><li class="chapter-item expanded "><div><strong aria-hidden="true">5.</strong> Developer guide</div></li><li><ol class="section"><li class="chapter-item expanded "><div><strong aria-hidden="true">5.1.</strong> Design</div></li></ol></li><li class="chapter-item expanded "><a href="faq.html">FAQ</a></li><li class="chapter-item expanded affix "><a href="links.html">Links</a></li><li class="chapter-item expanded affix "><a href="acknowledgments.html">Acknowledgments</a></li></ol>';
        // Set the current, active page, and reveal it if it's hidden
        let current_page = document.location.href.toString().split("#")[0];
        if (current_page.endsWith("/")) {
            current_page += "index.html";
        }
        var links = Array.prototype.slice.call(this.querySelectorAll("a"));
        var l = links.length;
        for (var i = 0; i < l; ++i) {
            var link = links[i];
            var href = link.getAttribute("href");
            if (href && !href.startsWith("#") && !/^(?:[a-z+]+:)?\/\//.test(href)) {
                link.href = path_to_root + href;
            }
            // The "index" page is supposed to alias the first chapter in the book.
            if (link.href === current_page || (i === 0 && path_to_root === "" && current_page.endsWith("/index.html"))) {
                link.classList.add("active");
                var parent = link.parentElement;
                if (parent && parent.classList.contains("chapter-item")) {
                    parent.classList.add("expanded");
                }
                while (parent) {
                    if (parent.tagName === "LI" && parent.previousElementSibling) {
                        if (parent.previousElementSibling.classList.contains("chapter-item")) {
                            parent.previousElementSibling.classList.add("expanded");
                        }
                    }
                    parent = parent.parentElement;
                }
            }
        }
        // Track and set sidebar scroll position
        this.addEventListener('click', function(e) {
            if (e.target.tagName === 'A') {
                sessionStorage.setItem('sidebar-scroll', this.scrollTop);
            }
        }, { passive: true });
        var sidebarScrollTop = sessionStorage.getItem('sidebar-scroll');
        sessionStorage.removeItem('sidebar-scroll');
        if (sidebarScrollTop) {
            // preserve sidebar scroll position when navigating via links within sidebar
            this.scrollTop = sidebarScrollTop;
        } else {
            // scroll sidebar to current active section when navigating via "next/previous chapter" buttons
            var activeSection = document.querySelector('#sidebar .active');
            if (activeSection) {
                activeSection.scrollIntoView({ block: 'center' });
            }
        }
        // Toggle buttons
        var sidebarAnchorToggles = document.querySelectorAll('#sidebar a.toggle');
        function toggleSection(ev) {
            ev.currentTarget.parentElement.classList.toggle('expanded');
        }
        Array.from(sidebarAnchorToggles).forEach(function (el) {
            el.addEventListener('click', toggleSection);
        });
    }
}
window.customElements.define("mdbook-sidebar-scrollbox", MDBookSidebarScrollbox);
