use crate::regret::RegretTable;

#[derive(Debug,Clone,PartialEq)]
pub struct ZeroSumMatrixGame{
    row_actions:usize,
    col_actions:usize,
    row_payoffs:Vec<f64>,
}

#[derive(Debug,Clone,Copy,PartialEq)]
pub struct MatrixGameReport{
    pub value:f64,
    pub row_best_response_value:f64,
    pub row_value_vs_col_best_response:f64,
    pub row_br_gain:f64,
    pub col_br_gain:f64,
    pub nashconv:f64,
}

impl ZeroSumMatrixGame{
    pub fn new(row_actions:usize,col_actions:usize,row_payoffs:Vec<f64>)->Result<Self,String>{
        if row_actions<2||col_actions<2{return Err("matrix game requires at least two actions per player".into());}
        if row_payoffs.len()!=row_actions*col_actions{return Err("matrix shape mismatch".into());}
        if row_payoffs.iter().any(|v|!v.is_finite()){return Err("matrix payoff must be finite".into());}
        Ok(Self{row_actions,col_actions,row_payoffs})
    }

    pub fn row_actions(&self)->usize{self.row_actions}
    pub fn col_actions(&self)->usize{self.col_actions}
    pub fn payoff(&self,row:usize,col:usize)->f64{self.row_payoffs[row*self.col_actions+col]}

    pub fn row_action_values(&self,col_strategy:&[f64])->Result<Vec<f64>,String>{
        validate_strategy(col_strategy,self.col_actions)?;
        let mut out=vec![0.0;self.row_actions];
        for r in 0..self.row_actions{
            for c in 0..self.col_actions{out[r]+=col_strategy[c]*self.payoff(r,c);}
        }
        Ok(out)
    }

    pub fn col_action_values(&self,row_strategy:&[f64])->Result<Vec<f64>,String>{
        validate_strategy(row_strategy,self.row_actions)?;
        let mut out=vec![0.0;self.col_actions];
        for c in 0..self.col_actions{
            for r in 0..self.row_actions{out[c]+=row_strategy[r]*(-self.payoff(r,c));}
        }
        Ok(out)
    }

    pub fn expected_row_value(&self,row_strategy:&[f64],col_strategy:&[f64])->Result<f64,String>{
        validate_strategy(row_strategy,self.row_actions)?;
        validate_strategy(col_strategy,self.col_actions)?;
        let mut value=0.0;
        for r in 0..self.row_actions{for c in 0..self.col_actions{value+=row_strategy[r]*col_strategy[c]*self.payoff(r,c);}}
        Ok(value)
    }

    pub fn evaluate(&self,row_strategy:&[f64],col_strategy:&[f64])->Result<MatrixGameReport,String>{
        let value=self.expected_row_value(row_strategy,col_strategy)?;
        let row_values=self.row_action_values(col_strategy)?;
        let row_br=row_values.into_iter().fold(f64::NEG_INFINITY,f64::max);

        // Column best response minimizes row player's value.
        validate_strategy(row_strategy,self.row_actions)?;
        let mut col_min=f64::INFINITY;
        for c in 0..self.col_actions{
            let mut row_value=0.0;
            for r in 0..self.row_actions{row_value+=row_strategy[r]*self.payoff(r,c);}
            col_min=col_min.min(row_value);
        }

        let row_gain=(row_br-value).max(0.0);
        let col_gain=(value-col_min).max(0.0);
        Ok(MatrixGameReport{
            value,
            row_best_response_value:row_br,
            row_value_vs_col_best_response:col_min,
            row_br_gain:row_gain,
            col_br_gain:col_gain,
            nashconv:row_gain+col_gain,
        })
    }

    /// One simultaneous regret-matching sweep. Both players' action values are
    /// computed from frozen pre-update strategies so update order cannot leak.
    pub fn simultaneous_sweep(&self,row:&mut RegretTable,col:&mut RegretTable)->Result<(),String>{
        if row.infosets()!=1||col.infosets()!=1{return Err("matrix harness expects one infoset per player".into());}
        if row.actions()!=self.row_actions||col.actions()!=self.col_actions{return Err("regret table action count does not match matrix".into());}
        let row_strategy=row.current_strategy(0);
        let col_strategy=col.current_strategy(0);
        let row_values=self.row_action_values(&col_strategy)?;
        let col_values=self.col_action_values(&row_strategy)?;
        row.update_from_action_values(0,&row_values,1.0,1.0);
        col.update_from_action_values(0,&col_values,1.0,1.0);
        Ok(())
    }
}

fn validate_strategy(strategy:&[f64],actions:usize)->Result<(),String>{
    if strategy.len()!=actions{return Err("strategy action count mismatch".into());}
    if strategy.iter().any(|p|!p.is_finite()||*p<0.0||*p>1.0){return Err("invalid strategy probability".into());}
    let sum=strategy.iter().sum::<f64>();
    if (sum-1.0).abs()>1e-9{return Err(format!("strategy sums to {sum}, expected 1"));}
    Ok(())
}

#[cfg(test)]
mod tests{
    use super::*;

    #[test]
    fn matching_pennies_uniform_is_exact_equilibrium(){
        let g=ZeroSumMatrixGame::new(2,2,vec![1.0,-1.0,-1.0,1.0]).unwrap();
        let r=g.evaluate(&[0.5,0.5],&[0.5,0.5]).unwrap();
        assert!(r.value.abs()<1e-12);
        assert!(r.nashconv.abs()<1e-12);
    }

    #[test]
    fn asymmetric_known_equilibrium_is_exact(){
        // [[4,0],[-1,2]] has row mix p=3/7, column mix q=2/7,
        // and value 8/7.
        let g=ZeroSumMatrixGame::new(2,2,vec![4.0,0.0,-1.0,2.0]).unwrap();
        let r=g.evaluate(&[3.0/7.0,4.0/7.0],&[2.0/7.0,5.0/7.0]).unwrap();
        assert!((r.value-8.0/7.0).abs()<1e-12);
        assert!(r.nashconv<1e-12);
    }

    #[test]
    fn simultaneous_sweep_uses_frozen_strategies(){
        let g=ZeroSumMatrixGame::new(2,2,vec![4.0,0.0,-1.0,2.0]).unwrap();
        let mut row=RegretTable::new(1,2);
        let mut col=RegretTable::new(1,2);
        g.simultaneous_sweep(&mut row,&mut col).unwrap();
        // From uniform/uniform: row values [2,0.5], column values [-1.5,-1].
        // Thus next current strategies must favor row action 0 and col action 1.
        let rs=row.current_strategy(0);let cs=col.current_strategy(0);
        assert!(rs[0]>rs[1]);
        assert!(cs[1]>cs[0]);
    }
}
