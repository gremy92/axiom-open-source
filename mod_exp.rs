use clap::Parser;
use halo2_base::gates::{GateChip, GateInstructions, RangeChip, RangeInstructions};
use halo2_base::utils::ScalarField;
use halo2_base::AssignedValue;
#[allow(unused_imports)]
use halo2_base::{
    Context,
    QuantumCell::{Constant, Existing, Witness},
};
use halo2_scaffold::scaffold::cmd::Cli;
use halo2_scaffold::scaffold::run;
use serde::{Deserialize, Serialize};
use std::env::var;

use crate::bigint::{big_is_equal, big_less_than, FixedOverflowInteger, ProperCrtUint};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CircuitInput {
    pub x: String, // field element, but easier to deserialize as a string
    pub y: String,
    pub p: String 
}

fn some_algorithm_in_zk<F: ScalarField>(
    ctx: &mut Context<F>,
    input: CircuitInput,
    make_public: &mut Vec<AssignedValue<F>>,
) {
    let x = F::from_str_vartime(&input.x).expect("deserialize field element should not fail");
    let y = F::from_str_vartime(&input.y).expect("deserialize field element should not fail");
    let p = F::from_str_vartime(&input.p).expect("deserialize field element should not fail");

    let x = ctx.load_witness(x);
    let y = ctx.load_witness(y);
    let p = ctx.load_witness(p);

    make_public.push(x);
    make_public.push(y);
    make_public.push(p);

    let gate = GateChip::<F>::default();

    let lookup_bits =
        var("LOOKUP_BITS").unwrap_or_else(|_| panic!("LOOKUP_BITS not set")).parse().unwrap();

    let range: RangeChip<F> = RangeChip::default(lookup_bits);

    let ybits = gate.num_to_bits(ctx, y, 200);

    let mut vsq = Vec::new();

    let x_mod = range.div_mod_var(ctx, x, p, 250, 250);

    vsq.push(x_mod.1);

    let mut xsq = x_mod.1;

    for _i in 1..200 {
        xsq = gate.mul(ctx, xsq, xsq);
        let xsq_mod = range.div_mod_var(ctx, xsq, p, 250, 250);
        xsq = xsq_mod.1;
        vsq.push(xsq);
    }

    let mut ans = ctx.load_witness(F::from(1));
    
    for i in 0..200 {
        let in1 = gate.mul_add(ctx, vsq[i], ybits[i], Constant(F::from(1)));
        let in2 = gate.sub(ctx, in1, ybits[i]);
        let in3 = gate.mul(ctx, ans, in2);
        let in4 = range.div_mod_var(ctx, in3, p, 250, 250);
        ans = in4.1;
    }

    make_public.push(ans);

    println!("ans: {:?}", ans.value());

}

fn main() {
    env_logger::init();

    let args = Cli::parse();

    run(some_algorithm_in_zk, args);
}
