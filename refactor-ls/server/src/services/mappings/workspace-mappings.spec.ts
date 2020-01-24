import { mapOutputToCrateList } from "./workspace-mappings";
import * as assert from "assert";

describe("mapOutputToCrateList", () => {
    const stdout = "Crate:{\"crate_name\":\"closuretests\",\"is_test\":false,\"replacements\":[{\"byte_end\":31,\"byte_start\":28,\"char_end\":16,\"char_start\":13,\"file_name\":\"src/main.rs\",\"line_end\":1,\"line_start\":1,\"replacement\":\"Box<u32>\"},{\"byte_end\":53,\"byte_start\":52,\"char_end\":7,\"char_start\":6,\"file_name\":\"src/main.rs\",\"line_end\":3,\"line_start\":3,\"replacement\":\"Box::new(0)\"}],\"errors\":[]}\nCrate:{\"crate_name\":\"closuretests\",\"is_test\":true,\"replacements\":[{\"byte_end\":31,\"byte_start\":28,\"char_end\":16,\"char_start\":13,\"file_name\":\"src/main.rs\",\"line_end\":1,\"line_start\":1,\"replacement\":\"Box<u32>\"},{\"byte_end\":53,\"byte_start\":52,\"char_end\":7,\"char_start\":6,\"file_name\":\"src/main.rs\",\"line_end\":3,\"line_start\":3,\"replacement\":\"Box::new(0)\"}],\"errors\":[]}\n";

    it("", () => {

        let actual = mapOutputToCrateList(stdout);

        assert.equal(actual.length, 2);
    });
});