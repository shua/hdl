module fsm_serial(
    input clk,
    input in,
    input reset,    // Synchronous reset
    output reg [7:0] out_byte,
    output done
);
    
    parameter IDLE = 0, START = 1, DATA=2, STOP = 3, SEARCH = 4;
    reg [2:0] s, next;
    reg [3:0] data_read;
    
    always @(*) begin
        case (s)
            IDLE: next = ~in ? START : IDLE;
            START: next = DATA;
            DATA: next = data_read < 8 ? DATA : in ? STOP : SEARCH;
            STOP: next = ~in ? START : IDLE;
            SEARCH: next = in ? IDLE : SEARCH;
            default: next = 'x;
        endcase
    end
    
    always @(posedge clk) begin
        if (reset)
            s = IDLE;
        else begin
            s = next;
            if (s == START) begin
                data_read = 0;
                out_byte = 'x;
            end else if (s == DATA) begin
                data_read = data_read + 1;
                out_byte = {in, out_byte[7:1]};
            end else if (s == STOP) begin
                data_read = 'x;
                out_byte = out_byte;
            end else begin
                data_read = 'x;
                out_byte = 'x;
            end
        end
    end
    
    assign done = s == STOP;
endmodule
