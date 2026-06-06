use super::*;

pub(crate) fn render_html_slides_bytes(response: &CompileResponse, options: &Value) -> Vec<u8> {
    let theme_id = options
        .get("presentationTheme")
        .and_then(Value::as_str)
        .unwrap_or("corporate");
    let transition = options
        .get("presentationTransition")
        .and_then(Value::as_str)
        .unwrap_or("fade");

    let (bg, text_color, accent) = theme_colors(theme_id);
    let slides = build_html_slides(response, options);
    let html = render_html_slides_document(
        response,
        &slides,
        bg,
        text_color,
        accent,
        transition,
        theme_id,
    );
    html.into_bytes()
}

#[derive(Debug)]
struct HtmlSlide {
    title: String,
    lines: Vec<String>,
    layout: HtmlSlideLayout,
    notes: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum HtmlSlideLayout {
    Title,
    Section,
    Content,
}

fn theme_colors(theme_id: &str) -> (&'static str, &'static str, &'static str) {
    match theme_id {
        "minimal" => ("#ffffff", "#1e293b", "#275DA8"),
        "dark" => ("#0f172a", "#f1f5f9", "#0f766e"),
        "nature" => ("#1a3326", "#f0fdf4", "#4ade80"),
        "warm" => ("#2d1b0e", "#fefce8", "#f59e0b"),
        _ => ("#1f3a5f", "#ffffff", "#4b9cd3"), // corporate default
    }
}

fn build_html_slides(response: &CompileResponse, options: &Value) -> Vec<HtmlSlide> {
    let title_lines = export_metadata_lines(response, options)
        .into_iter()
        .filter(|line| !line.starts_with("Cover: "))
        .collect();
    let mut slides = vec![HtmlSlide {
        title: response.semantic.title.clone(),
        lines: title_lines,
        layout: HtmlSlideLayout::Title,
        notes: Vec::new(),
    }];

    let mut current_title = String::new();
    let mut current_lines: Vec<String> = Vec::new();
    let mut current_layout = HtmlSlideLayout::Content;
    let mut current_notes: Vec<String> = Vec::new();

    let flush = |title: &mut String,
                 lines: &mut Vec<String>,
                 layout: &mut HtmlSlideLayout,
                 notes: &mut Vec<String>,
                 slides: &mut Vec<HtmlSlide>| {
        if !lines.is_empty() || title != "Continued" {
            slides.push(HtmlSlide {
                title: std::mem::take(title),
                lines: std::mem::take(lines),
                layout: *layout,
                notes: std::mem::take(notes),
            });
        } else {
            title.clear();
            lines.clear();
            notes.clear();
        }
        *layout = HtmlSlideLayout::Content;
    };

    for block in &response.document_ast.blocks {
        match block {
            DocumentBlock::Heading { level, text, .. } if *level <= 2 => {
                flush(
                    &mut current_title,
                    &mut current_lines,
                    &mut current_layout,
                    &mut current_notes,
                    &mut slides,
                );
                current_title = text.clone();
            }
            DocumentBlock::Layout { directive, .. } if directive == "page-break" => {
                flush(
                    &mut current_title,
                    &mut current_lines,
                    &mut current_layout,
                    &mut current_notes,
                    &mut slides,
                );
                current_title = "Continued".to_string();
            }
            DocumentBlock::Layout {
                directive,
                options: _opts,
                settings,
                ..
            } if directive == "section-break" => {
                flush(
                    &mut current_title,
                    &mut current_lines,
                    &mut current_layout,
                    &mut current_notes,
                    &mut slides,
                );
                current_title = settings
                    .title
                    .clone()
                    .unwrap_or_else(|| "Section".to_string());
                current_layout = HtmlSlideLayout::Section;
                current_notes = slide_notes_from_options(settings);
            }
            DocumentBlock::Layout {
                directive,
                options: _opts,
                settings,
                ..
            } if directive == "slide" => {
                flush(
                    &mut current_title,
                    &mut current_lines,
                    &mut current_layout,
                    &mut current_notes,
                    &mut slides,
                );
                current_title = slide_title_from_options(_opts, settings);
                current_layout = match settings.layout.as_deref() {
                    Some("title" | "title-slide" | "cover") => HtmlSlideLayout::Title,
                    Some("section" | "section-divider" | "divider") => HtmlSlideLayout::Section,
                    _ => HtmlSlideLayout::Content,
                };
                current_notes = slide_notes_from_options(settings);
            }
            _ => {
                if current_title.is_empty() && current_lines.is_empty() {
                    current_title = "Document".to_string();
                }
                current_lines.extend(block_export_lines(block));
            }
        }
    }

    if !current_lines.is_empty() || current_title != "Continued" {
        slides.push(HtmlSlide {
            title: current_title,
            lines: current_lines,
            layout: current_layout,
            notes: current_notes,
        });
    }

    slides
}

fn render_html_slides_document(
    response: &CompileResponse,
    slides: &[HtmlSlide],
    bg: &str,
    text_color: &str,
    accent: &str,
    transition: &str,
    theme_id: &str,
) -> String {
    let title = escape_html(&response.semantic.title);
    let author = metadata_string(&response.metadata, "author").unwrap_or_default();
    let slide_html: String = slides
        .iter()
        .enumerate()
        .map(|(i, slide)| render_slide_div(slide, i, slides.len(), bg, text_color, accent))
        .collect();
    let notes_json = render_notes_json(slides);
    let titles_json = render_titles_json(slides);
    let lines_json = render_lines_json(slides);
    let transition_css = transition_css(transition);

    format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8"/>
<meta name="viewport" content="width=device-width,initial-scale=1"/>
<title>{title}</title>
<meta name="author" content="{author_escaped}"/>
<meta name="generator" content="NEditor {version}"/>
<meta name="presentation-theme" content="{theme_id}"/>
<style>
*,*::before,*::after{{box-sizing:border-box;margin:0;padding:0}}
:root{{
  --bg:{bg};
  --text:{text_color};
  --accent:{accent};
  --slide-w:100vw;
  --slide-h:100vh;
}}
html,body{{width:100%;height:100%;overflow:hidden;background:var(--bg);color:var(--text);font-family:system-ui,sans-serif}}
#deck{{width:100%;height:100%;position:relative;overflow:hidden}}
.slide{{
  position:absolute;inset:0;
  display:flex;flex-direction:column;justify-content:center;align-items:flex-start;
  padding:6vh 8vw;
  opacity:0;pointer-events:none;
  {transition_css}
}}
.slide.active{{opacity:1;pointer-events:auto}}
.slide.layout-title{{align-items:center;text-align:center;justify-content:center}}
.slide.layout-section{{justify-content:center;background:color-mix(in srgb,var(--accent) 20%,var(--bg) 80%)}}
.slide-title{{font-size:clamp(1.4rem,4vw,3rem);font-weight:800;line-height:1.15;margin-bottom:0.6em;color:var(--text)}}
.slide.layout-title .slide-title{{font-size:clamp(1.8rem,5vw,4rem);border-bottom:3px solid var(--accent);padding-bottom:0.3em}}
.slide.layout-section .slide-title{{font-size:clamp(1.5rem,4vw,3.2rem);color:var(--accent)}}
.slide-body{{font-size:clamp(0.85rem,1.8vw,1.3rem);line-height:1.65;width:100%;max-width:900px}}
.slide-body ul{{padding-left:1.4em;margin-top:0.4em}}
.slide-body li{{margin-bottom:0.3em}}
.slide-body p{{margin-bottom:0.5em}}
.slide-num{{position:absolute;bottom:14px;right:20px;font-size:11px;opacity:0.4}}
#progress{{position:fixed;top:0;left:0;height:3px;background:var(--accent);transition:width 0.25s ease;z-index:200}}
#controls{{position:fixed;bottom:0;left:0;right:0;display:flex;align-items:center;gap:8px;padding:8px 14px;background:rgba(0,0,0,0.35);z-index:200;opacity:0;transition:opacity 0.2s}}
#deck:hover #controls{{opacity:1}}
#controls button{{background:rgba(255,255,255,0.12);border:1px solid rgba(255,255,255,0.2);color:rgba(255,255,255,0.85);border-radius:5px;padding:5px 12px;cursor:pointer;font-size:12px}}
#controls button:hover{{background:rgba(255,255,255,0.22)}}
#slide-counter{{flex:1;text-align:center;color:rgba(255,255,255,0.5);font-size:12px}}
#presenter-panel{{
  position:fixed;inset:0;z-index:500;background:#07101c;
  display:none;grid-template-columns:1fr 260px;grid-template-rows:1fr auto;gap:12px;padding:14px;
}}
#presenter-panel.open{{display:grid}}
#presenter-main{{display:flex;flex-direction:column;gap:10px;overflow:hidden}}
#presenter-slide-preview{{
  flex:1;border-radius:10px;overflow:hidden;
  display:flex;align-items:center;justify-content:center;min-height:0;
  background:var(--bg);color:var(--text);border:1px solid rgba(255,255,255,0.08);
}}
#presenter-slide-preview .preview-inner{{padding:28px 36px;width:100%}}
#presenter-slide-preview h2{{font-size:clamp(1rem,2.5vw,1.8rem);font-weight:750;margin-bottom:12px;border-bottom:2px solid rgba(255,255,255,0.2);padding-bottom:8px}}
#presenter-slide-preview ul{{font-size:clamp(0.75rem,1.4vw,1rem);padding-left:1.3em;line-height:1.6}}
#presenter-notes{{flex:0 0 110px;display:flex;flex-direction:column;gap:4px}}
.presenter-label{{font-size:9px;font-weight:750;text-transform:uppercase;letter-spacing:0.06em;color:#475569}}
#presenter-notes textarea{{flex:1;background:#111b29;border:1px solid #1e3050;border-radius:6px;color:#94a3b8;font:12px/1.5 inherit;padding:7px 9px;resize:none}}
#presenter-side{{display:flex;flex-direction:column;gap:10px;overflow:hidden}}
#presenter-next-box{{border-radius:8px;height:72px;overflow:hidden;border:1px solid #1e3050;display:flex;align-items:center;padding:10px 14px;background:var(--bg);font-size:12px;opacity:0.7;color:var(--text)}}
#presenter-slide-list{{flex:1;overflow-y:auto;display:flex;flex-direction:column;gap:2px}}
.pli{{display:flex;align-items:center;gap:6px;padding:4px 7px;border-radius:5px;cursor:pointer;color:#475569;font-size:11px}}
.pli:hover{{background:#1a2535;color:#64748b}}
.pli.active{{background:#152640;color:#7eaedd}}
.pli-num{{flex:0 0 18px;text-align:right;font-size:10px}}
.pli-title{{overflow:hidden;text-overflow:ellipsis;white-space:nowrap}}
#presenter-nav{{grid-column:1/3;display:flex;align-items:center;gap:10px;background:#111b29;border-radius:8px;padding:9px 14px}}
#presenter-nav button{{padding:5px 14px;border-radius:6px;background:#1e2d42;border:1px solid #2a4060;color:#90c0f0;cursor:pointer;font-weight:650;font-size:12px}}
#presenter-nav button:disabled{{opacity:0.4;cursor:not-allowed}}
#presenter-nav span{{flex:1;text-align:center;color:#64748b;font-size:12px}}
#presenter-timer{{font-variant-numeric:tabular-nums;color:#94a3b8;font-size:12px}}
</style>
</head>
<body>
<div id="progress"></div>
<div id="deck">
{slide_html}
</div>
<div id="controls">
  <button id="btn-prev" title="Previous slide (←)">← Prev</button>
  <span id="slide-counter">1 / {total}</span>
  <button id="btn-next" title="Next slide (→)">Next →</button>
  <button id="btn-fullscreen" title="Fullscreen (F)">⛶ Full</button>
  <button id="btn-presenter" title="Presenter view (P)">⊞ Present</button>
</div>
<div id="presenter-panel">
  <div id="presenter-main">
    <div id="presenter-slide-preview">
      <div class="preview-inner">
        <h2 id="pp-title"></h2>
        <ul id="pp-bullets"></ul>
      </div>
    </div>
    <div id="presenter-notes">
      <span class="presenter-label">Speaker notes</span>
      <textarea id="pp-notes" placeholder="No speaker notes for this slide."></textarea>
    </div>
  </div>
  <div id="presenter-side">
    <div>
      <span class="presenter-label">Next slide</span>
      <div id="presenter-next-box"></div>
    </div>
    <span class="presenter-label">Slides</span>
    <div id="presenter-slide-list"></div>
  </div>
  <div id="presenter-nav">
    <button id="pp-prev">← Prev</button>
    <span id="pp-counter">1 / {total}</span>
    <button id="pp-next">Next →</button>
    <span id="presenter-timer">0:00</span>
    <button id="pp-close" style="background:#1e0f1a;border-color:#4a1020;color:#f87171">✕ Close</button>
  </div>
</div>
<script>
(function(){{
  const NOTES = {notes_json};
  const TITLES = {titles_json};
  const LINES = {lines_json};
  var total = {total};
  var current = 0;
  var timerStart = null;
  var timerInterval = null;
  var slidesCache = Array.from(document.querySelectorAll('.slide'));

  function getSlides(){{ return slidesCache; }}

  function goTo(idx){{
    if(idx < 0 || idx >= total) return;
    var slides = getSlides();
    slides[current].classList.remove('active');
    current = idx;
    slides[current].classList.add('active');
    updateUI();
    updatePresenter();
  }}

  function updateUI(){{
    document.getElementById('slide-counter').textContent = (current+1) + ' / ' + total;
    document.getElementById('progress').style.width = ((current+1)/total*100) + '%';
    var prev = document.getElementById('btn-prev');
    var next = document.getElementById('btn-next');
    if(prev) prev.disabled = current === 0;
    if(next) next.disabled = current === total - 1;
  }}

  function buildPresenterList(){{
    var list = document.getElementById('presenter-slide-list');
    if(!list) return;
    list.innerHTML = '';
    TITLES.forEach(function(t, i){{
      var div = document.createElement('div');
      div.className = 'pli' + (i === current ? ' active' : '');
      div.innerHTML = '<span class="pli-num">' + (i+1) + '</span><span class="pli-title">' + escHtml(t||'(untitled)') + '</span>';
      div.addEventListener('click', function(){{ goTo(i); }});
      list.appendChild(div);
    }});
  }}

  function escHtml(s){{
    return s.replace(/&/g,'&amp;').replace(/</g,'&lt;').replace(/>/g,'&gt;');
  }}

  function updatePresenter(){{
    var ppTitle = document.getElementById('pp-title');
    var ppBullets = document.getElementById('pp-bullets');
    var ppNotes = document.getElementById('pp-notes');
    var ppNext = document.getElementById('presenter-next-box');
    var ppCounter = document.getElementById('pp-counter');
    if(ppTitle) ppTitle.textContent = TITLES[current] || '';
    if(ppBullets){{
      var lines = LINES[current] || [];
      ppBullets.innerHTML = lines.slice(0,8).map(function(l){{return '<li>'+escHtml(l)+'</li>';}}).join('');
    }}
    if(ppNotes) ppNotes.value = NOTES[current] || '';
    if(ppNext) ppNext.textContent = TITLES[current+1] || '(end of presentation)';
    if(ppCounter) ppCounter.textContent = (current+1) + ' / ' + total;
    var items = document.querySelectorAll('#presenter-slide-list .pli');
    items.forEach(function(item, i){{
      item.classList.toggle('active', i === current);
    }});
    var ppPrev = document.getElementById('pp-prev');
    var ppNextBtn = document.getElementById('pp-next');
    if(ppPrev) ppPrev.disabled = current === 0;
    if(ppNextBtn) ppNextBtn.disabled = current >= total - 1;
  }}

  function openPresenter(){{
    var panel = document.getElementById('presenter-panel');
    if(!panel) return;
    panel.classList.add('open');
    buildPresenterList();
    updatePresenter();
    if(!timerStart){{
      timerStart = Date.now();
      timerInterval = setInterval(function(){{
        var elapsed = Math.floor((Date.now() - timerStart) / 1000);
        var m = Math.floor(elapsed/60);
        var s = elapsed % 60;
        var el = document.getElementById('presenter-timer');
        if(el) el.textContent = m + ':' + (s < 10 ? '0' : '') + s;
      }}, 1000);
    }}
  }}

  function closePresenter(){{
    var panel = document.getElementById('presenter-panel');
    if(panel) panel.classList.remove('open');
    if(timerInterval){{ clearInterval(timerInterval); timerInterval = null; timerStart = null; }}
    var el = document.getElementById('presenter-timer');
    if(el) el.textContent = '0:00';
  }}

  document.addEventListener('keydown', function(e){{
    var inPresenter = document.getElementById('presenter-panel').classList.contains('open');
    if(e.key === 'ArrowRight' || e.key === 'ArrowDown' || e.key === ' '){{ e.preventDefault(); goTo(current+1); }}
    else if(e.key === 'ArrowLeft' || e.key === 'ArrowUp'){{ e.preventDefault(); goTo(current-1); }}
    else if(e.key === 'Escape' && inPresenter){{ closePresenter(); }}
    else if(e.key === 'p' || e.key === 'P'){{ if(!inPresenter) openPresenter(); }}
    else if(e.key === 'f' || e.key === 'F'){{ if(!inPresenter) toggleFullscreen(); }}
  }});

  function toggleFullscreen(){{
    if(!document.fullscreenElement) document.documentElement.requestFullscreen().catch(function(){{}});
    else document.exitFullscreen().catch(function(){{}});
  }}

  var btnPrev = document.getElementById('btn-prev');
  var btnNext = document.getElementById('btn-next');
  var btnFull = document.getElementById('btn-fullscreen');
  var btnPres = document.getElementById('btn-presenter');
  if(btnPrev) btnPrev.addEventListener('click', function(){{ goTo(current-1); }});
  if(btnNext) btnNext.addEventListener('click', function(){{ goTo(current+1); }});
  if(btnFull) btnFull.addEventListener('click', toggleFullscreen);
  if(btnPres) btnPres.addEventListener('click', openPresenter);

  var ppPrev = document.getElementById('pp-prev');
  var ppNext = document.getElementById('pp-next');
  var ppClose = document.getElementById('pp-close');
  if(ppPrev) ppPrev.addEventListener('click', function(){{ goTo(current-1); }});
  if(ppNext) ppNext.addEventListener('click', function(){{ goTo(current+1); }});
  if(ppClose) ppClose.addEventListener('click', closePresenter);

  // Init first slide
  var slides = getSlides();
  if(slides.length > 0) slides[0].classList.add('active');
  updateUI();
}})();
</script>
</body>
</html>"#,
        title = title,
        author_escaped = escape_html(&author),
        version = escape_html(env!("CARGO_PKG_VERSION")),
        theme_id = escape_html(theme_id),
        bg = escape_html(bg),
        text_color = escape_html(text_color),
        accent = escape_html(accent),
        transition_css = transition_css,
        slide_html = slide_html,
        total = slides.len(),
        notes_json = notes_json,
        titles_json = titles_json,
        lines_json = lines_json,
    )
}

fn transition_css(transition: &str) -> &'static str {
    match transition {
        "fade" => "transition:opacity 0.4s ease",
        "push" => "transition:opacity 0.3s ease,transform 0.3s ease",
        "wipe" => "transition:opacity 0.3s ease,clip-path 0.35s ease",
        "zoom" => "transition:opacity 0.35s ease,transform 0.35s ease",
        _ => "transition:none",
    }
}

fn render_slide_div(
    slide: &HtmlSlide,
    index: usize,
    total: usize,
    _bg: &str,
    _text_color: &str,
    accent: &str,
) -> String {
    let layout_class = match slide.layout {
        HtmlSlideLayout::Title => "layout-title",
        HtmlSlideLayout::Section => "layout-section",
        HtmlSlideLayout::Content => "layout-content",
    };
    let title_html = escape_html(&slide.title);
    let body_html: String = slide
        .lines
        .iter()
        .filter(|line| !line.trim().is_empty())
        .take(14)
        .map(|line| format!("<li>{}</li>", escape_html(line)))
        .collect();
    let body_html = if body_html.is_empty() {
        String::new()
    } else {
        format!("<ul>{}</ul>", body_html)
    };
    let slide_num = format!(
        r#"<span class="slide-num">{} / {}</span>"#,
        index + 1,
        total
    );
    let accent_bar = if slide.layout == HtmlSlideLayout::Title || slide.layout == HtmlSlideLayout::Section {
        format!(
            r#"<div style="width:60px;height:4px;background:{};border-radius:2px;margin-bottom:1em"></div>"#,
            escape_html(accent)
        )
    } else {
        String::new()
    };
    format!(
        r#"<div class="slide {layout_class}" data-index="{index}">
{accent_bar}<h2 class="slide-title">{title_html}</h2>
<div class="slide-body">{body_html}</div>
{slide_num}
</div>
"#
    )
}

fn render_notes_json(slides: &[HtmlSlide]) -> String {
    let notes: Vec<String> = slides.iter().map(|s| s.notes.join(" ")).collect();
    serde_json::to_string(&notes).unwrap_or_else(|_| "[]".to_string())
}

fn render_titles_json(slides: &[HtmlSlide]) -> String {
    let titles: Vec<&str> = slides.iter().map(|s| s.title.as_str()).collect();
    serde_json::to_string(&titles).unwrap_or_else(|_| "[]".to_string())
}

fn render_lines_json(slides: &[HtmlSlide]) -> String {
    let lines: Vec<&[String]> = slides.iter().map(|s| s.lines.as_slice()).collect();
    serde_json::to_string(&lines).unwrap_or_else(|_| "[]".to_string())
}
