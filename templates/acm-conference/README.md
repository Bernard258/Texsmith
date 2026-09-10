# ACM conference layout

Uses `acmart` with `sigconf,nonacm` for a local two-column draft without invented ACM publication metadata. Before an ACM submission, follow the venue instructions: review submissions may require `manuscript,review,anonymous`; proceedings production requires removing `nonacm`, adding the assigned rights/conference/DOI metadata, and providing real CCS concepts. `acmart` supplies graphics, tables, math fonts, and citation packages.

Sources: [acmart package and documentation](https://ctan.org/pkg/acmart), [ACM author instructions](https://authors.acm.org/proceedings/production-information/preparing-your-article-with-latex).

Edit the authors and abstract in `main.tex`, the paper text in
`sections/body.tex`, and citations in `references.bib`. Replace the example
citation and illustrative table. Add images to `figures/`.

Compile from the Reports workspace with `texsmith build REPORT-NAME`.
Tectonic runs BibTeX and the required LaTeX passes automatically. The class
and bibliography style are loaded from its TeX bundle on demand; no separate
class download is needed. These are editable starters, not submission guarantees.

If Tectonic omits `build/main.log` because the class reads it during compilation,
run `tectonic --keep-intermediates --keep-logs --outdir build main.tex` from
inside the report folder to retain diagnostic logs and auxiliary files.
