set terminal png size 1920,1080
set output '05_enemies.png'
plot '05_enemies_base.dat' title 'base' with points pt 7, '05_enemies_2000.dat' title 'leaves=2000', '05_enemies_8000.dat' title 'leaves=8000', '05_enemies_20000.dat' title 'leaves=20000', '05_enemies_50000.dat' title 'leaves=50000', '05_enemies_100000.dat' title 'leaves=100000', '05_enemies_200000.dat' title 'leaves=200000', '05_enemies_500000.dat' title 'leaves=500000'
