`timescale 1 ns / 1 ns
module test;
reg [7:0] out_byte;
reg clk, reset, in;
wire done;

fsm_serial test (clk, in, reset, out_byte, done);

parameter HALF_PERIOD = 10, PERIOD = HALF_PERIOD*2;
initial begin
  $dumpvars();
  reset = 1;
  in = 0;
  clk = 1;

  #PERIOD     reset = 0;
  #(PERIOD*9) in = 1;
  #PERIOD     in = 0;
  $monitor("%t: out_byte = %08h, done = %d", $time, out_byte, done);
  #PERIOD $finish;
end

always #HALF_PERIOD clk = ~clk;

endmodule
