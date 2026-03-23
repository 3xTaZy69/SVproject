module main;

    logic x = 0;

    initial begin
        case (x + 1)
            4'b0001: $display("TRUE");
            default: $display("FALSE");
        endcase

        $dumpfile("wave.vcd");
        $dumpvars(0, main);

        $display("Hello, SV!");

        #1;
        $finish;
    end

endmodule