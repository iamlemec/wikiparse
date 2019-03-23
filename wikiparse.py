# extract specific articles from wiki database
# fast parse: Liza Daly, http://www.ibm.com/developerworks/xml/library/x-hiperfparse/

import sys
import bz2
import time
import argparse
import pandas as pd
from lxml import etree

# parse input arguments
parser = argparse.ArgumentParser(description='Extract desired page ids from wiki XML.')
parser.add_argument('--input', type=str, help='wiki xml source file')
parser.add_argument('--output', type=str, default=None, help='wiki xml output file')
parser.add_argument('--pages', type=str, default=None, help='csv of page ids')
parser.add_argument('--log', type=int, default=None, help='log file name')
args = parser.parse_args()

wiki_header = b"""<mediawiki xmlns="http://www.mediawiki.org/xml/export-0.10/" xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance" xsi:schemaLocation="http://www.mediawiki.org/xml/export-0.10/ http://www.mediawiki.org/xml/export-0.10.xsd" version="0.10" xml:lang="en">"""
wiki_footer = b'</mediawiki>'

namespace = '{http://www.mediawiki.org/xml/export-0.10/}'
page_tag = namespace + 'page'
id_tag = namespace + 'id'
title_tag = namespace + 'title'

fin = bz2.open(args.input, 'r')

fout = open(args.output, 'wb+') if args.output is not None else None
if fout:
    fout.write(wiki_header+b'\n')

flog = open(args.log, 'a+', 1) if args.log is not None else sys.stdout
flog.write(f'{args.input} -> {args.output}\n')

pages = list(pd.read_csv(args.pages)['id']) if args.pages is not None else None

art_tot = 0
hit_tot = 0
time0 = time.time()

try:
    for event, elem in etree.iterparse(fin, tag=page_tag, events=['end'], recover=True):
        id = int(elem.find(id_tag).text)
        if pages is None or id in pages:
            hit_tot += 1
            try:
                if fout:
                    etree.ElementTree(elem).write(fout, encoding='UTF-8', pretty_print=True)
                else:
                    print(elem.find(title_tag).text)
            except Exception as err:
                flog.write(f'{err}\n')
                flog.write(f'failed to write {id}, line {elem.sourceline}\n')

        art_tot += 1
        if art_tot % 50 == 0:
            time1 = time.time()
            flog.write(f'articles {art_tot}, matches {hit_tot}, id {id}, time {time1-time0}\n')

        elem.clear()
        while elem.getprevious() is not None:
            del elem.getparent()[0]
except KeyboardInterrupt:
    print()
except Exception as e:
    raise

print(art_tot)
print(hit_tot)

fout.write(wiki_footer+b'\n')
flog.write('\n')
