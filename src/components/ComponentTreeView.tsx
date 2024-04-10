import { invoke } from "@tauri-apps/api";
import { ask, message } from "@tauri-apps/api/dialog";
import { MouseEventHandler, useCallback, useState } from "react";
import { IoMdArrowDropright } from "react-icons/io";
import { useDispatch, useSelector } from "react-redux";
import { setSelected } from "../redux/state/stateSlice";
import { RootState } from "../redux/store";
import { Category } from "../types";
import { Checkbox } from "./Checkbox";
import { readableByteSize } from "../util";

enum NodeState {
  CHECKED,
  INDETERMINATE,
  UNCHECKED
}

type NodeRender = {
  render: JSX.Element;
  combinedState: NodeState;
}

export function ComponentsTreeView() {
  const { appState } = useSelector((state: RootState) => state.state);
  const { selected, required } = appState.components;
  const dispatch = useDispatch();
  const [expanded, setExpanded] = useState<string[]>([]);
  console.log(required);

  const renderCategoryNode = useCallback((category: Category, level: number = 0): NodeRender => {
    const isExpanded = expanded.includes(category.id);
    let combinedState: NodeState = NodeState.CHECKED;
    let totalSelected = 0;
    const children: JSX.Element[] = [];
    for (const subcat of category.subcategories) {
      const nodeRender = renderCategoryNode(subcat, level + 1);
      if (nodeRender.combinedState !== NodeState.UNCHECKED) {
        totalSelected += 1;
      }
      if (nodeRender.combinedState !== NodeState.CHECKED) {
        combinedState = NodeState.INDETERMINATE
      }
      if (isExpanded) {
        children.push(nodeRender.render);
      }
    }
    for (const component of category.components) {
      const compSelected = selected.includes(component.id) || required.includes(component.id);
      if (compSelected) {
        totalSelected += 1;
      } else {
        combinedState = NodeState.INDETERMINATE;
      }
      if (isExpanded) {
        children.push(
          <div
            id={component.id}
            className="tree-node"
            onClick={(event) => {
              event.stopPropagation();
              toggleComponent(component.id);
            }}
            style={{marginLeft: `${(level + 2)}rem`}}>
            <Checkbox
              disabled={required.includes(component.id)}
              checked={selected.includes(component.id) || required.includes(component.id)}/>
            {`${component.name} - ${readableByteSize(component.install_size)}`}
          </div>
        );
      }
    }
    if (totalSelected === 0) {
      combinedState = NodeState.UNCHECKED;
    }
    const render = (
      <div
        id={category.id}
        className="tree-node"
        style={{marginLeft: `${(level)}rem`}}
        onClick={(event) => {
          console.log('select cat');
          event.stopPropagation();
          // If CHECKED, 'false' for 'I do not want to be checked'
          toggleComponent(category.id, combinedState !== NodeState.CHECKED);
        }}
        >
        <ArrowIcon 
          onClick={(event) => {
            event.stopPropagation();
            console.log('clicked ' + category.id);
            const newExpanded = [...expanded];
            const expId = newExpanded.findIndex(e => e === category.id);
            if (expId > -1) {
              newExpanded.splice(expId);
            } else {
              newExpanded.push(category.id);
            }
            setExpanded(newExpanded);
          }}
          isOpen={isExpanded}/>
        <Checkbox
          disabled={required.includes(category.id)}
          indeterminate={combinedState === NodeState.INDETERMINATE}
          checked={required.includes(category.id) || combinedState !== NodeState.UNCHECKED} />
        {category.name}
        {isExpanded && (
          <>
            {children}
          </>
        )}
      </div>
    )
    return {
      render,
      combinedState
    };
  }, [expanded, selected, required]);

  const toggleComponent = async (id: string, newState?: boolean): Promise<boolean> => {
    console.log(id);
    const checked = newState === undefined ? !selected.includes(id) : newState; // Desired checked state
    try {
      if (!checked) {
        console.log('unselect');
        // unselecting, make sure we want to remove all dependants too
        let dependants: string[] = await invoke("find_component_dependants", { id });
        console.log(dependants);
        dependants = dependants.filter(d => d !== id && selected.includes(d));
        if (dependants.length > 0) {
          // Confirm before disabling
          ask(`Unselecting "${id}" will also unselect ${dependants.length} other components (${dependants.join(', ')}). Is this okay?`)
          .then((success) => {
            if (success) {
              invoke('unselect_component', { id })
              .catch((error) => {
                message(error, 'Error');
              });
              return true;
            }
          })
          .catch((error) => {
            message(error, 'Error');
          });
        } else {
          const compIdx = appState.components.selected.findIndex(c => c === id);
          if (compIdx > -1) {
            invoke('unselect_component', { id })
            .catch((error) => {
              message(error, 'Error');
            });
            return true;
          }
        }
      } else {
        // selecting, just enable it, backend will get all dependencies
        dispatch(setSelected([...appState.components.selected, id]));
        invoke('select_component', { id })
        .catch((error) => {
          message(error, 'Error');
        });
        return true;
      }
    } catch (error) {
      message(`${error}`);
    }
    return false;
  }

  // Event handlers and additional logic here...

  const catRenders = appState.components.categories.map((cat) => renderCategoryNode(cat, 0));
  return (
    <div>
      {catRenders.map(c => c.render)}
    </div>
  );
}

type ArrowIconProps = {
  isOpen: boolean;
  className?: string;
  onClick?: MouseEventHandler<SVGElement>
}

const ArrowIcon = ({ isOpen, className, onClick }: ArrowIconProps) => {
  const baseClass = "arrow";
  const classes = `${baseClass} ${isOpen ? `${baseClass}--open` : `${baseClass}--closed`} ${className}`;
  return <IoMdArrowDropright onClick={onClick} className={classes} />;
};