/* example program */
module main; /* module automatically spawns if its name matches file name */

    wire clk; /* by default, value = 0 */

    initial begin /* executes in initial cycle only */
        fixed(0, 0, 0) clk; /* place node at exact coordinates
         only decimal are allowed and
         expressions are restricted 
         in itinial

        only this type of comments is allowed
        */
        tick(5) clk; /*
            makes clk tick every 5 ticks
            clk has to be 1 bit wide and never assigned
        */
    end

endmodule