CC:=latexmk
OPTIONS:=-pdf --shell-escape
TARGET:=paper

all:
	$(CC) $(OPTIONS) $(TARGET)

clean_partial:
	rm -f $(TARGET).aux
	rm -f $(TARGET).bcf-SAVE-ERROR
	rm -f $(TARGET).bbl
	rm -f $(TARGET).blg
	rm -f $(TARGET).dvi
	rm -f $(TARGET).fdb_latexmk
	rm -f $(TARGET).fls
	rm -f $(TARGET).glo
	rm -f $(TARGET).hd
	rm -f $(TARGET).idx
	rm -f $(TARGET).ins
	rm -f $(TARGET).log
	rm -f $(TARGET).out
	rm -f $(TARGET).toc
	rm -f $(TARGET).run.xml
	rm -f $(TARGET).bcf
	rm -f archive.zip

clean: clean_partial
	rm -f $(TARGET).pdf

fresh: clean all

zip: fresh
	$(MAKE) clean_partial
	zip archive.zip *.sty
	zip archive.zip *.dtx
	zip archive.zip *.tex
	zip archive.zip *.pdf
	zip archive.zip -r figures/
	zip archive.zip -r docs/
