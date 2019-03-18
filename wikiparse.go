package main

import (
	"compress/bzip2"
	"encoding/xml"
	"encoding/csv"
	"fmt"
	"io"
	"os"
	"strconv"
    "time"
	"flag"
	"log"
)

// A Page in the wiki.
type Page struct {
	Title     string     `xml:"title"`
	ID        uint64     `xml:"id"`
	Ns        uint64     `xml:"ns"`
}

// Generic text
type Elem struct {
	Inner  string   `xml:",innerxml"`
}

func GetInner(dec *xml.Decoder, tag *xml.StartElement) (string) {
  	elem := &Elem{}
  	err := dec.DecodeElement(elem, tag)
  	if err == nil {
    	return elem.Inner
  	} else {
    	return ""
  	}
}

func ParseUint64(num string) uint64 {
  	x, _ := strconv.ParseUint(num, 10, 64)
  	return x
}

func parse(inp_fname string, out_fname string, id_list map[uint64]bool, limit int) (error) {
	// open input file for reading
	finp, err := os.Open(inp_fname)
	if err != nil {
		log.Fatal("Input XML file not found: ", err)
		return err
	}
	defer finp.Close()

	// construct decoder with bz2
	bzr := bzip2.NewReader(finp)
	dec := xml.NewDecoder(bzr)

	// open output file for writing
	fout, err := os.Create(out_fname)
	if err != nil {
		log.Fatal("Couldn't create output file: ", err)
		return err
	}
	defer fout.Close()

	// parse header info
	dec.Token() // read mediawiki
	site := Elem{} // store siteinfo
	dec.Decode(&site) // read siteinfo

	// write header info
	fout.WriteString("<mediawiki>\n")
	fout.WriteString("  <siteinfo>")
	fout.WriteString(site.Inner);
	fout.WriteString("</siteinfo>\n")

  	// get zero time
  	time0 := time.Now().Unix()

  	// init memory
  	page := &Page{}
	start := false
	done := false
	total := 0
	hits := 0

  	// parse all pages
	for {
		if done {
			fout.WriteString("</mediawiki>\n")
			break
		}

		tok, err := dec.Token()
    	if err == io.EOF {
      		break
    	}

    	switch tag := tok.(type) {
    	case xml.StartElement:
      		switch tag.Name.Local {
      		case "page":
        		page = &Page{}
				start = false

				// limit parsing
				if limit > 0 && total >= limit {
					done = true
				}
      		case "title":
        		page.Title = GetInner(dec, &tag)
      		case "ns":
        		inner := GetInner(dec, &tag)
        		page.Ns = ParseUint64(inner)
      		case "id":
        		inner := GetInner(dec, &tag)
        		page.ID = ParseUint64(inner)

				// logging
				total += 1
				if total % 50 == 0 {
        			time1 := time.Now().Unix()
					fmt.Printf("articles %d, matches %d, id %d, time %d\n", total, hits, page.ID, time1-time0)
				}

				// article match
				if id_list[page.ID] {
					hits += 1
					fmt.Printf("%s\n", page.Title)
				} else {
					dec.Skip()
				}
			case "revision":
				if !start {
					fmt.Fprintf(fout, "  <page>\n    <title>%s</title>\n    <ns>%d</ns>\n    <id>%d</id>\n", page.Title, page.Ns, page.ID)
				}
				start = true

				inner := GetInner(dec, &tag)
				fout.WriteString("    <revision>")
				fout.WriteString(inner)
				fout.WriteString("</revision>\n")
      		default:
        		dec.Skip()
      		}
    	case xml.EndElement:
      		switch tag.Name.Local {
      		case "page":
				fout.WriteString("  </page>\n")
      		case "mediawiki":
				done = true
      		}
    	}
  	}

	// return status
	return nil
}

// read in ids as a bool map
func readlist(fname string) (map[uint64]bool) {
	f, err := os.Open(fname)
	if err != nil {
		log.Fatal("Article file not found")
	}

	r := csv.NewReader(f)
	r.Read() // clear header

	ids := map[uint64]bool{}
	for {
		rec, err := r.Read()
		if err == io.EOF {
			break
		}
		if len(rec) > 0 {
			i := ParseUint64(rec[0])
			ids[i] = true
		}
	}

	return ids
}

func main() {
	var inp_fname = flag.String("input", "", "input filename (bz2)")
	var out_fname = flag.String("output", "", "output filename (xml)")
	var id_fname = flag.String("articles", "", "articles filename (csv)")
	var limit = flag.Int("limit", 0, "limit number of pages")
	flag.Parse()

	id_list := readlist(*id_fname)
	parse(*inp_fname, *out_fname, id_list, *limit)
}
