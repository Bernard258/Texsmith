# IEEE journal

Uses `IEEEtran` in `journal` mode with two columns, author affiliation footnotes, index terms, and IEEE references. Specific journals may require additional class options or their own template.

Source: [IEEEtran documentation](https://www.michaelshell.org/tex/ieeetran/).

Edit the authors and abstract in `main.tex`, the paper text in
`sections/body.tex`, and citations in `references.bib`. Replace the example
citation and illustrative table. Add images to `figures/`.

Compile from the Reports workspace with `texsmith build REPORT-NAME`.
Tectonic runs BibTeX and the required LaTeX passes automatically. The class
and bibliography style are loaded from its TeX bundle on demand; no separate
class download is needed. These are editable starters, not submission guarantees.

The template loads T1 font encoding before the class so Tectonic uses the
Times fonts selected by IEEEtran rather than falling back to Latin Modern.
