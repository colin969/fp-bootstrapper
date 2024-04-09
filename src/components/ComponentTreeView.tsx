import { SimpleTreeView, TreeItem } from "@mui/x-tree-view";
import { Category, Component, ComponentList } from "../types";
import { useContext, useMemo } from "react";
import { Checkbox, FormControlLabel } from "@mui/material";
import { ask, message } from "@tauri-apps/api/dialog";
import { invoke } from "@tauri-apps/api";
import { StateContext } from "../StateContext";

export type ComponentsTreeViewProps = {
  list: ComponentList;
}

export function ComponentsTreeView(props: ComponentsTreeViewProps) {
  const { setState, state } = useContext(StateContext);

  const toggleComponent = async (component: Component, checked: boolean) => {
    try {
      if (!checked) {
        // unselecting, make sure we want to remove all dependants too
        const dependants: Component[] = await invoke("find_component_dependants", { id: component.id });
        if (dependants.length > 0) {
          // Confirm before disabling
          ask(`Unselecting "${component.title}" will also unselect ${dependants.length} other components (${dependants.map(c => c.title).join(', ')}). Is this okay?`)
          .then((success) => {
            if (success) {
              const newState = {...state};
              const compIdx = newState.components.selected.findIndex(c => c === component.id);
              if (compIdx > -1) {
                newState.components.selected.splice(compIdx, 1);
                setState(newState);
                invoke('unselect_component', { id: component.id })
                .catch((error) => {
                  message(error, 'Error');
                });
              }
            }
          })
          .catch((error) => {
            message(error, 'Error');
          });
        } else {
          const newState = {...state};
          const compIdx = newState.components.selected.findIndex(c => c === component.id);
          if (compIdx > -1) {
            newState.components.selected.splice(compIdx, 1);
            setState(newState);
            invoke('unselect_component', { id: component.id })
            .catch((error) => {
              message(error, 'Error');
            });
          }
        }
      } else {
        // Add change to frontend to avoid input delay
        const newState = {...state};
        newState.components.selected.push(component.id);
        setState(newState);
        // selecting, just enable it, backend will get all dependencies
        invoke('select_component', { id: component.id })
        .catch((error) => {
          message(error, 'Error');
        });
      }
    } catch (error) {
      message(`${error}`);
    }
  }

  const renderTreeComponent = (component: Component) => {
    return (
      <TreeItem key={component.id} itemId={component.real_id} label={
          <FormControlLabel
            onChange={(_, checked) => toggleComponent(component, checked)}
            checked={state.components.selected.includes(component.id)}
            label={`${component.real_id} - ${readableByteSize(component.install_size)}`}
            control={<Checkbox/>} />
          }>
      </TreeItem>
    );
  }

  const renderTreeCategory = (category: Category) => {
    let checked = false;
    let indeterminate = false;
    for (const component of category.components) {
      if (state.components.selected.includes(component.id)) { checked = true; }
      if (!state.components.selected.includes(component.id) && checked) {
        indeterminate = true;
        break;
      }
    }
    return (
      <TreeItem key={category.id} itemId={category.real_id} label={
          <FormControlLabel
            label={category.real_id}
            control={
              <Checkbox checked={checked} indeterminate={indeterminate} />
            }/>
          }>
        {category.subcategories ? category.subcategories.map(renderTreeCategory) : undefined}
        {category.components ? category.components.map(renderTreeComponent) : undefined}
      </TreeItem>
    );
  }

  const renderComponents = useMemo(() => {
    return props.list.categories.map(renderTreeCategory);
  }, [props.list]);

  // Event handlers and additional logic here...

  return (
    <SimpleTreeView>
      {renderComponents}
    </SimpleTreeView>
  );
}

const thresh = 1024;

function readableByteSize(bytes: number) {
  if (Math.abs(bytes) < thresh) {
    return bytes + ' B';
  }

  const units = ['KiB', 'MiB', 'GiB', 'TiB', 'PiB', 'EiB', 'ZiB', 'YiB'];
  let u = -1;
  const r = 10**1;

  do {
    bytes /= thresh;
    ++u;
  } while (Math.round(Math.abs(bytes) * r) / r >= thresh && u < units.length - 1);


  return bytes.toFixed(1) + ' ' + units[u];
}
