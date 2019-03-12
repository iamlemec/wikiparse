package main

import (
	"compress/bzip2"
	"encoding/xml"
	"encoding/csv"
	"fmt"
	"io"
	"os"
	"bufio"
	"strconv"
    "time"
)

// SiteInfo is the toplevel site info describing basic dump properties.
type SiteInfo struct {
	SiteName   string `xml:"sitename"`
	Base       string `xml:"base"`
	Generator  string `xml:"generator"`
	Case       string `xml:"case"`
	Namespaces []struct {
		Key   string `xml:"key,attr"`
		Case  string `xml:"case,attr"`
		Value string `xml:",chardata"`
	} `xml:"namespaces>namespace"`
	Inner	  string     `xml:",innerxml"`
}

// A Page in the wiki.
type Page struct {
	Title     string     `xml:"title"`
	ID        uint64     `xml:"id"`
	// Redir     Redirect   `xml:"redirect"`
	// Revisions []Revision `xml:"revision"`
	Ns        uint64     `xml:"ns"`
	// Inner	  string     `xml:",innerxml"`
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

func parse(inp_fname string, out_fname string, id_list map[uint64]bool) (error) {
	// open input file for reading
	finp, err := os.Open(inp_fname)
	if err != nil {
		fmt.Println("Input XML file not found")
		return err
	}
	defer finp.Close()

	// construct decoder with bz2
	bzr := bzip2.NewReader(finp)
	dec := xml.NewDecoder(bzr)

	// open output file for writing
	fout, err := os.Create(out_fname)
	if err != nil {
		return err
	}
	defer fout.Close()

	// construct buffered writer
	buf := bufio.NewWriter(fout)

	// start root tag
	buf.WriteString("<mediawiki>\n")

	// parse opening mediawiki tag
	_, err = dec.Token()
	if err != nil {
		return err
	}

	// get siteinfo struct
	si := SiteInfo{}
	err = dec.Decode(&si)
	if err != nil {
		return err
	}

	// store siteinfo in output
	buf.WriteString("  <siteinfo>")
	buf.WriteString(si.Inner);
	buf.WriteString("</siteinfo>\n")

  	// get zero time
  	time0 := time.Now().Unix()

  	// init memory
  	page := &Page{}
	start := false
	total := 0
	hits := 0

  	// get all pages
	for {
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
				} else {
					dec.Skip()
				}
			case "revision":
				if !start {
					buf.WriteString("  <page>\n")
					fmt.Fprintf(buf, "    <title>%s</title>\n", page.Title)
					fmt.Fprintf(buf, "    <ns>%d</ns>\n", page.Ns)
					fmt.Fprintf(buf, "    <id>%d</id>\n", page.ID)
				}
				start = true

				inner := GetInner(dec, &tag)
				buf.WriteString("    <revision>")
				buf.WriteString(inner)
				buf.WriteString("</revision>\n")
      		default:
        		dec.Skip()
      		}
    	case xml.EndElement:
      		switch tag.Name.Local {
      		case "page":
				buf.WriteString("  </page>\n")
      		case "mediawiki":
				buf.WriteString("</mediawiki>\n")
      		}
    	}
  	}

	// return status
	return nil
}

// read in ids as a bool map
func readlist(fname string) (map[uint64]bool) {
	f, _ := os.Open(fname)
	b := bufio.NewReader(f)
	r := csv.NewReader(b)

	ids := map[uint64]bool{}

	_, err := r.Read() // clear header
	if err != nil {
		fmt.Println("No data found")
		return ids
	}

	for {
		rec, err := r.Read()
		if err == io.EOF {
			break
		} else if err != nil {
			fmt.Println("Error reading data")
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
	inp_fname := os.Args[1]
	out_fname := os.Args[2]
	id_fname := os.Args[3]

	id_list := readlist(id_fname)
	parse(inp_fname, out_fname, id_list)
}
