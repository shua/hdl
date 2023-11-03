SRC = \
	fsm_serial.v
TEST = \
	fsm_serial_tb.v

.SUFFIXES: .v .svg .dot .vcd .dsn .fst

all: fsm_serial_tb.fst

.v.dot:
	@(\
		echo 'read -sv $<;  hierarchy -auto-top;'; \
		if [ -z "$(NO_OPT)" ]; then \
			echo 'proc; opt; memory; opt; fsm; opt;'; \
		else \
			echo 'proc; opt; fsm;'; \
		fi; \
		echo 'show -prefix '"$$(basename -s .v "$<")"' -format dot'; \
	) | yosys -q

.dot.svg:
	dot -Tsvg $< -o $@

.v.dsn:
	iverilog -g2012 -o $@ $< $(SRC)
.dsn.fst:
	vvp $< -fst
	mv dump.fst $@

.PHONY: clean
clean:
	rm -f *.svg *.dot *.vcd *.fst
