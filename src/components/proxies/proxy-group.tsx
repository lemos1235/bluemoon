import { Box, SxProps, Theme } from "@mui/material"
import ProxyList from "./proxy-list";
import { Plus } from 'lucide-react'
import { useRef, useState } from "react";
import { ProxyEditDialog, ProxyEditDialogRef } from "./proxy-edit-dialog";
import { useMoon } from "@/hooks/use-moon";

//本地节点
export const LocalProxies = () => {

  const viewerRef = useRef<ProxyEditDialogRef>(null);

  const { moon } = useMoon();

  let current: string | undefined = moon?.current_proxy_id;

  let nodeList: IMoonProxy[] = [];

  //群组ID为"0"的是本地节点
  const localGroup = moon?.proxy_group_list?.find((e) => {
    return e.uid = "0";
  })
  if (localGroup?.proxy_list) {
    nodeList = localGroup.proxy_list;
  }

  if (nodeList.length === 0) {
    return (
      <Box sx={{ display: "flex", alignItems: "center", justifyContent: "center", height: "100%" }}>
        <Box sx={{
          color: "var(--text-primary)", cursor: "pointer", marginTop: "-80px",
          display: "flex", alignItems: "center", justifyContent: "center",
        }} onClick={() => viewerRef.current?.create()}>
          <Plus size={36} /><span style={{ fontSize: "18px", marginLeft: "5px" }}>新增节点</span>
        </Box>
        <ProxyEditDialog ref={viewerRef} />
      </Box>
    )
  }

  return (
    <Box>
      <Box sx={{
        color: "var(--text-primary)", cursor: "pointer",
        textAlign: "right", marginBottom: "-10px", marginRight: "15px",
      }} onClick={() => viewerRef.current?.create()}>
        <Plus />
      </Box>
      <ProxyList current={current} nodeList={nodeList} />
      <ProxyEditDialog ref={viewerRef} />
    </Box>
  )
}

//订阅节点
export const SubscriptionProxies = () => {
  const groups: IMoonProxyGroup[] = [];
  return (
    <Box>
      订阅节点
    </Box>
  )
}
