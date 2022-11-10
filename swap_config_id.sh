#!/usr/bin/bash

state=`grep -E 'set _CPUTAPID 0x[12]ba01477' black_pill.cfg | sed 's/set _CPUTAPID 0x\([12]\)ba01477/\1/g'`

case $state in
    1)
        sed -i 's/\(set _CPUTAPID 0x\)[12]\(ba01477\)/\12\2/g' black_pill.cfg 
        ;;
    2)
        sed -i 's/\(set _CPUTAPID 0x\)[12]\(ba01477\)/\11\2/g' black_pill.cfg 
        ;;
    *)
        ;;
esac
