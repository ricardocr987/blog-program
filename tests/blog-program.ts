import * as anchor from "@project-serum/anchor";
import { Program } from "@project-serum/anchor";
import { BlogProgram } from "../target/types/blog_program";

describe("blog-program", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.Provider.env());

  const program = anchor.workspace.BlogProgram as Program<BlogProgram>;

  it("Is initialized!", async () => {
    // Add your test here.
    const tx = await program.rpc.initialize({});
    console.log("Your transaction signature", tx);
  });
});
