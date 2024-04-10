export type CheckboxProps = {
  checked: boolean;
  indeterminate?: boolean;
  disabled?: boolean;
}

export function Checkbox(props: CheckboxProps) {
  return (
    <input
      readOnly
      disabled={props.disabled}
      className="tree-node-checkbox"
      type="checkbox"
      title={props.indeterminate ? props.indeterminate.toString() : 'coc'}
      ref={el => el && (el.indeterminate = !!props.indeterminate)}
      checked={props.checked}/>
  )
}