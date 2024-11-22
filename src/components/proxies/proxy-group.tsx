import { useMoon } from "@/hooks/use-moon";
import { Circle } from "@mui/icons-material";
import {
  Box,
  IconButton,
  List,
  ListItem,
  Menu,
  MenuItem,
  Stack,
} from "@mui/material";
import { useLockFn } from "ahooks";
import { MoreVertical, Plus, RefreshCcw } from "lucide-react";
import { useRef, useState } from "react";
import { BaseAlertDialog } from "../base/base-alert-dialog";
import { ProxyEditDialog, ProxyEditDialogRef } from "./proxy-edit-dialog";
import {
  ProxyGroupEditDialog,
  ProxyGroupEditDialogRef,
} from "./proxy-group-edit-dialog";
import ProxyList from "./proxy-list";
import { refreshSubscription } from "@/services/sub";
import { Notice } from "../base";

//本地节点
export const LocalProxies = () => {
  const proxyEditDialogRef = useRef<ProxyEditDialogRef>(null);

  const { moon } = useMoon();

  //群组ID为"0"的是本地节点
  const localGroup = moon?.proxy_group_list?.find((e) => e.uid === "0");
  const nodeList: IMoonProxy[] = localGroup?.proxy_list || [];

  if (nodeList.length === 0) {
    return (
      <Box
        sx={{
          display: "flex",
          alignItems: "center",
          justifyContent: "center",
          height: "100%",
        }}
      >
        <Box
          sx={{
            color: "var(--text-primary)",
            cursor: "pointer",
            marginTop: "-25px",
            display: "flex",
            alignItems: "center",
            justifyContent: "center",
          }}
          onClick={() => proxyEditDialogRef.current?.create()}
        >
          <Plus size={36} />
          <span style={{ fontSize: "18px", marginLeft: "8px" }}>本地节点</span>
        </Box>
        <ProxyEditDialog ref={proxyEditDialogRef} />
      </Box>
    );
  }

  return (
    <Box sx={{ height: "100%", overflowY: "auto" }}>
      <Box
        sx={{
          color: "var(--text-primary)",
          cursor: "pointer",
          textAlign: "right",
          padding: "8px 8px 0 0",
          marginBottom: "-8px",
        }}
        onClick={() => proxyEditDialogRef.current?.create()}
      >
        <Plus />
      </Box>
      <ProxyList nodeList={nodeList} />
      <ProxyEditDialog ref={proxyEditDialogRef} />
    </Box>
  );
};

interface SubscriptionStatusProps {
  isActived: boolean;
}

const SubscriptionStatus = (props: SubscriptionStatusProps) => {
  const { isActived } = props;
  return (
    <Circle
      sx={{ fontSize: "8px", marginLeft: "8px" }}
      color={isActived ? "primary" : "disabled"}
    />
  );
};

interface SubscriptionRefreshButtonProps {
  onClick: () => void;
}

const SubscriptionRefreshButton = (props: SubscriptionRefreshButtonProps) => {
  const { onClick } = props;

  return (
    <IconButton onClick={onClick}>
      <RefreshCcw size={18} />
    </IconButton>
  );
};

interface SubscriptionPopupMenuProps {
  onEdit: () => void;
  onDelete: () => void;
}

const SubscriptionPopupMenu = (props: SubscriptionPopupMenuProps) => {
  const { onEdit, onDelete } = props;
  const [anchorEl, setAnchorEl] = useState<null | HTMLElement>(null);
  const open = Boolean(anchorEl);

  const handleClick = (event: React.MouseEvent<HTMLElement>) => {
    setAnchorEl(event.currentTarget);
  };

  const handleClose = () => {
    setAnchorEl(null);
  };

  const handleEdit = () => {
    setAnchorEl(null);
    onEdit();
  };

  const handleDelete = () => {
    setAnchorEl(null);
    onDelete();
  };

  return (
    <Box marginLeft={"4px"}>
      <IconButton onClick={handleClick}>
        <MoreVertical size={20} />
      </IconButton>
      <Menu anchorEl={anchorEl} open={open} onClose={handleClose}>
        <MenuItem sx={{ width: "80px", fontSize: "14px" }} onClick={handleEdit}>
          编辑
        </MenuItem>
        <MenuItem
          sx={{ width: "80px", fontSize: "14px" }}
          onClick={handleDelete}
        >
          删除
        </MenuItem>
      </Menu>
    </Box>
  );
};

interface SubscriptionTitleProps {
  group: IMoonProxyGroup;
  onEdit: () => void;
  onDelete: () => void;
}

const SubscriptionTitle = (props: SubscriptionTitleProps) => {
  const { group, onEdit, onDelete } = props;

  const { saveProxyGroup } = useMoon();

  const fetchSubscription = useLockFn(async () => {
    try {
      const newGroup = await refreshSubscription(group);
      await saveProxyGroup(newGroup);
    } catch (err: any) {
      console.error(err);
      Notice.error("获取订阅失败");
    }
  });

  return (
    <Box sx={{ padding: "0 0 0 15px", marginBottom: "-8px" }}>
      <Stack direction="row" justifyContent="space-between">
        <Stack direction="row" alignItems={"center"}>
          <span>{group.remark || group.name}</span>
        </Stack>
        <Stack direction="row" alignItems={"center"}>
          {group.url && (
            <SubscriptionRefreshButton onClick={fetchSubscription} />
          )}
          <SubscriptionPopupMenu onEdit={onEdit} onDelete={onDelete} />
        </Stack>
      </Stack>
    </Box>
  );
};

interface SubscriptionContentProps {
  group: IMoonProxyGroup;
}

const SubscriptionContent = (props: SubscriptionContentProps) => {
  const { group } = props;

  const proxyGroupEditDialogRef = useRef<ProxyGroupEditDialogRef>(null);

  const [deleteOpen, setDeleteOpen] = useState(false);
  const [alertOpen, setAlertOpen] = useState(false);

  const { moon, deleteProxyGroup } = useMoon();

  const openDelete = () => {
    //检查当前节点是否关联某个规则
    const isRelated =
      moon?.rule_list?.some((r) =>
        group.proxy_list?.some((p) => p.uid === r.action)
      ) ?? false;
    if (isRelated) {
      setAlertOpen(true);
    } else {
      setDeleteOpen(true);
    }
  };

  const handleDelete = useLockFn(async () => {
    await deleteProxyGroup(group);
    setDeleteOpen(false);
  });

  return (
    <Stack width={"100%"}>
      <SubscriptionTitle
        group={group}
        onEdit={() => proxyGroupEditDialogRef.current?.edit(group)}
        onDelete={() => openDelete()}
      />
      <ProxyList nodeList={group.proxy_list || []} />
      <BaseAlertDialog
        open={deleteOpen}
        title="警告"
        message="是否删除此订阅组？"
        onClose={() => setDeleteOpen(false)}
        onConfirm={handleDelete}
      />
      <BaseAlertDialog
        open={alertOpen}
        title="警告"
        message="已有规则关联到此订阅组"
        onConfirm={() => setAlertOpen(false)}
      />
      <ProxyGroupEditDialog ref={proxyGroupEditDialogRef} />
    </Stack>
  );
};

//订阅节点
export const SubscriptionProxies = () => {
  const proxyGroupEditDialogRef = useRef<ProxyGroupEditDialogRef>(null);

  const { moon } = useMoon();

  let groups: IMoonProxyGroup[] =
    moon?.proxy_group_list?.filter((e) => e.uid !== "0") || [];

  if (groups.length === 0) {
    return (
      <Box
        sx={{
          display: "flex",
          alignItems: "center",
          justifyContent: "center",
          height: "100%",
        }}
      >
        <Box
          sx={{
            color: "var(--text-primary)",
            cursor: "pointer",
            marginTop: "-25px",
            display: "flex",
            alignItems: "center",
            justifyContent: "center",
          }}
          onClick={() => proxyGroupEditDialogRef.current?.create()}
        >
          <Plus size={36} />
          <span style={{ fontSize: "18px", marginLeft: "8px" }}>订阅节点</span>
        </Box>
        <ProxyGroupEditDialog ref={proxyGroupEditDialogRef} />
      </Box>
    );
  }

  return (
    <Box sx={{ height: "100%", overflowY: "auto" }}>
      <Box
        sx={{
          color: "var(--text-primary)",
          cursor: "pointer",
          textAlign: "right",
          padding: "8px 8px 0 0",
          marginBottom: "-8px",
        }}
        onClick={() => proxyGroupEditDialogRef.current?.create()}
      >
        <Plus />
      </Box>
      <List>
        {groups.map((group, index) => (
          <ListItem key={index} sx={{ padding: "0" }}>
            <SubscriptionContent group={group} />
          </ListItem>
        ))}
      </List>
      <ProxyGroupEditDialog ref={proxyGroupEditDialogRef} />
    </Box>
  );
};
