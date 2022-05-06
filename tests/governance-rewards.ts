import * as anchor from "@project-serum/anchor";
import { Program } from "@project-serum/anchor";
import { GovernanceRewards } from "../target/types/governance_rewards";

describe("governance-rewards", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.GovernanceRewards as Program<GovernanceRewards>;

  it("Is initialized!", async () => {
    // Add your test here.
    const tx = await program.methods.initialize().rpc();
    console.log("Your transaction signature", tx);
  });
});
