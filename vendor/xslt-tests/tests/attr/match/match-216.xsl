<?xml version="1.0" encoding="UTF-8"?>
<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" 
    xmlns:xs="http://www.w3.org/2001/XMLSchema" 
    xmlns:my="my" 
    exclude-result-prefixes="xs my" 
    version="3.0">
    
    <xsl:variable name="data">
      <A>
        <B>treasure</B>
      </A>
    </xsl:variable>

    <xsl:function name="my:test" as="xs:boolean">
        <xsl:param name="elt" as="node()" />
        <xsl:param name="ancestor-elt" as="element(*)" />
        <xsl:sequence select="true()"/>
    </xsl:function>

    <xsl:template match="A[B[my:test(., current())]]"><ok/></xsl:template>
    
    <xsl:template name="main"><xsl:apply-templates select="$data/*"/></xsl:template>

</xsl:stylesheet>