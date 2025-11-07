CC:=latexmk
OPTIONS:=-pdf --shell-escape
TARGET:=paper

all:
	$(CC) $(OPTIONS) $(TARGET)

clean_partial:
	rm -f *.aux
	rm -f *.bbl
	rm -f *.blg
	rm -f *.dvi
	rm -f *.fdb_latexmk
	rm -f *.fls
	rm -f *.glo
	rm -f *.hd
	rm -f *.idx
	rm -f *.ins
	rm -f *.log
	rm -f *.out
	rm -f *main.toc
	rm -f archive.zip
	rm -f *.xml
	rm -f *.bcf

clean: clean_partial
	rm -f *.pdf

fresh: clean all

zip: fresh
	$(MAKE) clean_partial
	zip archive.zip *.sty
	zip archive.zip *.dtx
	zip archive.zip *.tex
	zip archive.zip *.pdf
	zip archive.zip -r figures/
	zip archive.zip -r docs/
