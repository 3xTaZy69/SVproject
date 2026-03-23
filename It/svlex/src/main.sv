/* example program */
module main; /* module automatically spawns if its name matches file name */

    wire x; /* by default, value = 0 */

    initial begin /* executes in initial cycle only */
        fixed(0, 0, 0) x; /* place node at exact coordinates
         only decimal are allowed and
         expressions are restricted 
         in itinial

        only this type of comments is allowed
        */
    end

endmodule